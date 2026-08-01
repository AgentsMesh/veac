use std::collections::BTreeSet;

use veac_artifact::MediaRole;
use veac_plan::canonical::{AudioMixSource, SequenceId, SidechainSource, TrackId, TrackKind};
use veac_plan::{ResolvedAudioRoute, ResolvedClip, ResolvedClipSource, ResolvedRenderPlan};

use super::Usage;

#[derive(Clone, Copy)]
pub(in crate::emitter) enum Selection<'a> {
    Master,
    Track(&'a TrackId),
    Bus(&'a str),
}

impl<'a> From<&'a AudioMixSource> for Selection<'a> {
    fn from(value: &'a AudioMixSource) -> Self {
        match value {
            AudioMixSource::Master => Self::Master,
            AudioMixSource::Track { track_id } => Self::Track(track_id),
            AudioMixSource::Bus { bus_id } => Self::Bus(bus_id.as_str()),
        }
    }
}

pub(super) fn collect(plan: &ResolvedRenderPlan, selection: Selection<'_>, usage: &mut Usage) {
    visit(plan, selection, |clip| match &clip.source {
        ResolvedClipSource::Media {
            input_id,
            audio_stream: Some(_),
            ..
        } => usage.media(input_id, MediaRole::Audio),
        ResolvedClipSource::Multicam { source } => {
            for switch in &source.switches {
                if let Some(angle) = source
                    .angles
                    .iter()
                    .find(|angle| angle.id == switch.angle_id && angle.audio_stream.is_some())
                {
                    usage.media(&angle.input_id, MediaRole::Audio);
                }
            }
        }
        _ => {}
    });
}

pub(in crate::emitter) fn visit<'a>(
    plan: &'a ResolvedRenderPlan,
    selection: Selection<'_>,
    mut visitor: impl FnMut(&'a ResolvedClip),
) {
    let mut walker = Walker {
        plan,
        sequences: BTreeSet::new(),
        tracks: BTreeSet::new(),
        clips: BTreeSet::new(),
        visitor: &mut visitor,
    };
    walker.sequence(&plan.entry_sequence_id, selection);
}

struct Walker<'a, 'b, F> {
    plan: &'a ResolvedRenderPlan,
    sequences: BTreeSet<SequenceId>,
    tracks: BTreeSet<(SequenceId, TrackId, bool)>,
    clips: BTreeSet<(SequenceId, veac_plan::canonical::ItemId)>,
    visitor: &'b mut F,
}

impl<'a, F: FnMut(&'a ResolvedClip)> Walker<'a, '_, F> {
    fn sequence(&mut self, id: &SequenceId, selection: Selection<'_>) {
        if !self.sequences.insert(id.clone()) {
            return;
        }
        let Some(sequence) = self.plan.sequences.iter().find(|value| value.id == *id) else {
            return;
        };
        for track in &sequence.tracks {
            let selected = track.state.audio_enabled
                && match selection {
                    Selection::Master => matches!(track.kind, TrackKind::Video | TrackKind::Audio),
                    Selection::Track(id) => track.id == *id,
                    Selection::Bus(id) => route_is(track, id),
                };
            if selected {
                self.track(sequence, track, true);
            }
        }
    }

    fn track(
        &mut self,
        sequence: &'a veac_plan::ResolvedSequence,
        track: &'a veac_plan::ResolvedTrack,
        sidechains: bool,
    ) {
        if !self
            .tracks
            .insert((sequence.id.clone(), track.id.clone(), sidechains))
        {
            return;
        }
        for clip in &track.clips {
            if !crate::emitter::audio_source::can_build(clip) {
                continue;
            }
            if sidechains {
                self.sidechain(sequence, track, clip);
            }
            if self.clips.insert((sequence.id.clone(), clip.id.clone())) {
                (self.visitor)(clip);
                if let ResolvedClipSource::Sequence { sequence_id } = &clip.source {
                    self.sequence(sequence_id, Selection::Master);
                }
            }
        }
    }

    fn sidechain(
        &mut self,
        sequence: &'a veac_plan::ResolvedSequence,
        target: &'a veac_plan::ResolvedTrack,
        clip: &'a ResolvedClip,
    ) {
        let Some(source) = clip
            .audio
            .as_ref()
            .and_then(|audio| audio.sidechain.as_ref())
        else {
            return;
        };
        for track in &sequence.tracks {
            if track.id != target.id
                && track.state.audio_enabled
                && matches!(track.kind, TrackKind::Video | TrackKind::Audio)
                && source_matches(track, &source.source)
            {
                self.control_track(sequence, track, clip, source);
            }
        }
    }

    fn control_track(
        &mut self,
        sequence: &'a veac_plan::ResolvedSequence,
        track: &'a veac_plan::ResolvedTrack,
        target: &'a ResolvedClip,
        sidechain: &'a veac_plan::ResolvedSidechain,
    ) {
        let Some(active) = crate::emitter::audio_sidechain::active_window(target, sidechain) else {
            return;
        };
        for clip in track.clips.iter().filter(|clip| {
            crate::emitter::audio_source::can_build(clip) && intersects(clip.record_range, active)
        }) {
            if self.clips.insert((sequence.id.clone(), clip.id.clone())) {
                (self.visitor)(clip);
                if let ResolvedClipSource::Sequence { sequence_id } = &clip.source {
                    self.sequence(sequence_id, Selection::Master);
                }
            }
        }
    }
}

fn intersects(
    left: veac_plan::canonical::TimeRange,
    right: veac_plan::canonical::TimeRange,
) -> bool {
    left.end()
        .ok()
        .zip(right.end().ok())
        .is_some_and(|(left_end, right_end)| left.start < right_end && right.start < left_end)
}

fn source_matches(track: &veac_plan::ResolvedTrack, source: &SidechainSource) -> bool {
    match source {
        SidechainSource::Track { track_id } => track.id == *track_id,
        SidechainSource::Bus { bus_id } => route_is(track, bus_id.as_str()),
    }
}

fn route_is(track: &veac_plan::ResolvedTrack, id: &str) -> bool {
    matches!(&track.routing.audio, Some(ResolvedAudioRoute::Bus { bus_id }) if bus_id == id)
}
