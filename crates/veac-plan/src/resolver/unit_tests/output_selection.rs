use super::{mechanism_helpers::text_clip, support::*};
use crate::{canonical::*, resolve_one, ResolvedRenderPlan, ResolvedTrack};

#[test]
fn caption_sidecar_selects_exact_tracks_without_visual_applies() {
    let mut value = project();
    value.project.materials.push(font_material("med_font"));
    let first = text_clip(true);
    let mut second = first.clone();
    second.id = ItemId::new("itm_caption_b").unwrap();
    let selected_id = TrackId::new("trk_caption_a").unwrap();
    let sequence = &mut value.project.sequences[0];
    sequence.tracks.push(track(
        selected_id.as_str(),
        TrackKind::Caption,
        10,
        vec![first],
    ));
    sequence
        .tracks
        .push(track("trk_caption_b", TrackKind::Caption, 20, vec![second]));
    sequence.applies.push(apply(
        "apl_sidecar",
        ApplyTarget::Layer {
            track_id: selected_id.clone(),
        },
    ));
    value.project.render_configs[0].deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_sidecar").unwrap(),
        file_name: "captions.vtt".to_owned(),
        kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
            format: CaptionSidecarFormat::WebVtt,
            track_ids: vec![selected_id],
        }),
    }];

    let plan = resolve_one(&value, &RenderConfigId::new("out_main").unwrap()).unwrap();
    let selected = resolved_track(&plan, "trk_caption_a");
    assert!(selected.state.include_in_render);
    assert!(!selected.state.visual_enabled);
    assert_eq!(selected.clips.len(), 1);
    let omitted = resolved_track(&plan, "trk_caption_b");
    assert!(!omitted.state.include_in_render);
    assert!(omitted.clips.is_empty());
    assert!(plan.sequences[0].applies.is_empty());
}

#[test]
fn track_and_bus_stems_select_only_their_routed_tracks() {
    let mut value = stem_project();
    let config_id = value.project.render_configs[0].id.clone();
    let plan = resolve_one(&value, &config_id).unwrap();
    assert!(!resolved_track(&plan, "trk_video").state.include_in_render);
    assert!(resolved_track(&plan, "trk_audio_a").state.audio_enabled);
    assert!(!resolved_track(&plan, "trk_audio_b").state.include_in_render);

    value.project.sequences[0].tracks[2].routing = TrackRouting::AudioBus {
        bus_id: BusId::new("bus_music").unwrap(),
    };
    let DeliverableKind::AudioStem(settings) =
        &mut value.project.render_configs[0].deliverables[0].kind
    else {
        unreachable!();
    };
    settings.source = AudioStemSource::Bus {
        bus_id: BusId::new("bus_music").unwrap(),
    };
    let plan = resolve_one(&value, &config_id).unwrap();
    assert!(!resolved_track(&plan, "trk_audio_a").state.include_in_render);
    assert!(resolved_track(&plan, "trk_audio_b").state.audio_enabled);
}

fn stem_project() -> ProjectEnvelope {
    let mut value = project();
    value.project.materials.push(audio_material("med_audio"));
    let mut first = media_clip("itm_audio_a", "med_audio", 0);
    first.audio = Some(audio_properties());
    let mut second = media_clip("itm_audio_b", "med_audio", 0);
    second.audio = Some(audio_properties());
    let sequence = &mut value.project.sequences[0];
    sequence
        .tracks
        .push(track("trk_audio_a", TrackKind::Audio, 10, vec![first]));
    sequence
        .tracks
        .push(track("trk_audio_b", TrackKind::Audio, 20, vec![second]));
    value.project.render_configs[0].deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_stem").unwrap(),
        file_name: "dialogue.wav".to_owned(),
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec: AudioCodec::PcmS24Le,
                sample_rate: 48_000,
                channels: 2,
            },
            source: AudioStemSource::Track {
                track_id: TrackId::new("trk_audio_a").unwrap(),
            },
        }),
    }];
    value
}

fn resolved_track<'a>(plan: &'a ResolvedRenderPlan, id: &str) -> &'a ResolvedTrack {
    plan.sequences[0]
        .tracks
        .iter()
        .find(|track| track.id.as_str() == id)
        .unwrap()
}
