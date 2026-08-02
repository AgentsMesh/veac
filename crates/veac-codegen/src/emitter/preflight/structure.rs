use std::collections::BTreeSet;

use veac_plan::canonical::{Generator, ItemId, MaterialKind, SequenceId, TrackId, TrackKind};
use veac_plan::{ResolvedClipSource, ResolvedInputKind, ResolvedRenderPlan, ResolvedTrack};

use super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    let mut track_ids = BTreeSet::new();
    let mut item_ids = BTreeSet::new();
    for sequence in &plan.sequences {
        if SequenceId::new(sequence.id.as_str()).is_err()
            || sequence.name.trim().is_empty()
            || !veac_plan::canonical::render_geometry_valid(
                sequence.settings.width,
                sequence.settings.height,
                sequence.settings.frame_rate,
            )
            || !veac_plan::canonical::ffmpeg_sample_rate_valid(sequence.settings.sample_rate)
            || sequence.duration.timescale != plan.header.source.timebase
        {
            invalid(
                check,
                sequence.id.to_string(),
                "sequence identity or default untrusted-plan render budget is invalid",
            );
        }
        let mut orders = BTreeSet::new();
        let mut source_orders = BTreeSet::new();
        if !sequence
            .tracks
            .windows(2)
            .all(|pair| track_key(&pair[0]) < track_key(&pair[1]))
        {
            invalid(
                check,
                sequence.id.to_string(),
                "resolved tracks are not canonically ordered",
            );
        }
        for track in &sequence.tracks {
            if TrackId::new(track.id.as_str()).is_err()
                || !track_ids.insert(track.id.to_string())
                || !orders.insert(track.order)
                || !source_orders.insert(track.source_order)
                || !state_valid(track)
            {
                invalid(
                    check,
                    track.id.to_string(),
                    "track identity, order, state, or routing is invalid",
                );
            }
            let mut clip_orders = BTreeSet::new();
            if !track
                .clips
                .windows(2)
                .all(|pair| clip_key(&pair[0]) < clip_key(&pair[1]))
            {
                invalid(
                    check,
                    track.id.to_string(),
                    "resolved clips are not canonically ordered",
                );
            }
            for clip in &track.clips {
                if ItemId::new(clip.id.as_str()).is_err()
                    || !item_ids.insert(clip.id.to_string())
                    || !clip_orders.insert(clip.source_order)
                    || !clip_shape_valid(plan, track, clip)
                    || clip.record_range.start.value < 0
                    || clip.record_range.start.timescale != plan.header.source.timebase
                    || clip.record_range.duration.timescale != plan.header.source.timebase
                    || !clip
                        .record_range
                        .end()
                        .is_ok_and(|end| end <= sequence.duration)
                {
                    invalid(
                        check,
                        clip.id.to_string(),
                        "clip identity, order, range, or media shape is invalid",
                    );
                }
            }
        }
    }
}

fn track_key(track: &ResolvedTrack) -> (i32, u32, &str) {
    (track.order, track.source_order, track.id.as_str())
}

fn clip_key(clip: &veac_plan::ResolvedClip) -> (i64, u32, &str) {
    (
        clip.record_range.start.value,
        clip.source_order,
        clip.id.as_str(),
    )
}

fn state_valid(track: &ResolvedTrack) -> bool {
    let visual = track.state.visual_enabled
        && track.kind != TrackKind::Audio
        && track.routing.visual.is_some();
    let audio = track.state.audio_enabled
        && matches!(track.kind, TrackKind::Video | TrackKind::Audio)
        && track.routing.audio.is_some();
    let caption_only = track.state.include_in_render
        && track.kind == TrackKind::Caption
        && !track.state.visual_enabled
        && !track.state.audio_enabled;
    track.state.visual_enabled == visual
        && track.state.audio_enabled == audio
        && track.state.include_in_render == (visual || audio || caption_only)
        && (track.state.visual_enabled || track.routing.visual.is_none())
        && (track.state.audio_enabled || track.routing.audio.is_none())
        && (track.state.include_in_render
            || (track.clips.is_empty() && track.transitions.is_empty()))
        && (!matches!(&track.routing.audio, Some(veac_plan::ResolvedAudioRoute::Bus { bus_id }) if bus_id.trim().is_empty()))
}

fn clip_shape_valid(
    plan: &ResolvedRenderPlan,
    track: &ResolvedTrack,
    clip: &veac_plan::ResolvedClip,
) -> bool {
    let source_ok = match &clip.source {
        ResolvedClipSource::Media { input_id, .. } => plan.inputs.iter().any(|input| {
            input.id == *input_id
                && matches!(
                    (&input.kind, track.kind),
                    (
                        ResolvedInputKind::Media {
                            material_kind: MaterialKind::Video
                        },
                        TrackKind::Video | TrackKind::Visual | TrackKind::Audio
                    ) | (
                        ResolvedInputKind::Media {
                            material_kind: MaterialKind::Image
                        },
                        TrackKind::Video | TrackKind::Visual
                    ) | (
                        ResolvedInputKind::Media {
                            material_kind: MaterialKind::Audio
                        },
                        TrackKind::Audio
                    )
                )
        }),
        ResolvedClipSource::FreezeFrame { .. }
        | ResolvedClipSource::Sequence { .. }
        | ResolvedClipSource::Multicam { .. } => {
            matches!(track.kind, TrackKind::Video | TrackKind::Visual)
        }
        ResolvedClipSource::Text { .. } => track.kind == TrackKind::Visual,
        ResolvedClipSource::Caption { .. } => track.kind == TrackKind::Caption,
        ResolvedClipSource::Generated {
            generator: Generator::Silence,
        } => track.kind == TrackKind::Audio,
        ResolvedClipSource::Generated { .. } => {
            matches!(track.kind, TrackKind::Video | TrackKind::Visual)
        }
    };
    let (text_ok, styled) = match &clip.source {
        ResolvedClipSource::Text { content } => {
            (!content.text.is_empty() && content.styled().is_some(), true)
        }
        ResolvedClipSource::Caption { content, .. } => {
            (!content.text.is_empty(), content.styled().is_some())
        }
        _ => (true, false),
    };
    source_ok
        && text_ok
        && (clip.audio.is_none() || audio_capable(&clip.source))
        && clip.visual.is_some() == (track.state.visual_enabled || styled)
        && (clip.audio.is_none() || track.state.audio_enabled)
        && (track.kind != TrackKind::Audio || clip.visual.is_none())
        && (!matches!(track.kind, TrackKind::Visual | TrackKind::Caption) || clip.audio.is_none())
}

fn audio_capable(source: &ResolvedClipSource) -> bool {
    matches!(
        source,
        ResolvedClipSource::Media { .. }
            | ResolvedClipSource::Sequence { .. }
            | ResolvedClipSource::Multicam { .. }
            | ResolvedClipSource::Generated {
                generator: Generator::Silence
            }
    )
}

fn invalid(check: &mut Check, id: String, message: &'static str) {
    check.push("PLAN_STRUCTURE_INVALID", Some(id), message);
}
