use std::collections::{BTreeMap, BTreeSet};

use veac_ir::{
    Clip, ClipSource, ProjectEnvelope, RelationGraph, RenderConfig, SequenceId, TrackKind,
};

use crate::EffectiveTrackState;

use super::{
    demand::{SequenceDemand, TextDemand},
    ClipDemand,
};

type TrackStates = BTreeMap<(SequenceId, veac_ir::TrackId), EffectiveTrackState>;
type ClipDemands = BTreeMap<(SequenceId, veac_ir::ItemId), ClipDemand>;

pub(super) fn analyze(
    envelope: &ProjectEnvelope,
    config: &RenderConfig,
) -> (TrackStates, ClipDemands) {
    Walker::new(envelope, config).run()
}

struct Walker<'a> {
    envelope: &'a ProjectEnvelope,
    relations: RelationGraph<'a>,
    demands: BTreeMap<SequenceId, SequenceDemand>,
    pending: BTreeSet<SequenceId>,
    tracks: TrackStates,
    clips: ClipDemands,
}

impl<'a> Walker<'a> {
    fn new(envelope: &'a ProjectEnvelope, config: &RenderConfig) -> Self {
        let root = config.sequence_id.clone();
        Self {
            envelope,
            relations: RelationGraph::project(&envelope.project),
            demands: BTreeMap::from([(root.clone(), SequenceDemand::root(config))]),
            pending: BTreeSet::from([root]),
            tracks: BTreeMap::new(),
            clips: BTreeMap::new(),
        }
    }

    fn run(mut self) -> (TrackStates, ClipDemands) {
        while let Some(id) = self.pending.iter().next().cloned() {
            self.pending.remove(&id);
            self.sequence(&id);
        }
        (self.tracks, self.clips)
    }

    fn sequence(&mut self, id: &SequenceId) {
        let Some(sequence) = self
            .envelope
            .project
            .sequences
            .iter()
            .find(|sequence| sequence.id == *id)
        else {
            return;
        };
        let demand = self.demands.get(id).cloned().unwrap_or_default();
        let activity = veac_ir::SequenceActivity::new(sequence);
        for track in &sequence.tracks {
            let active = activity.track_live(track);
            let main_visual = active
                && demand.visual
                && matches!(track.kind, TrackKind::Video | TrackKind::Visual);
            let burn_caption = active && demand.burn_captions && track.kind == TrackKind::Caption;
            let control_visual = active
                && track
                    .clips
                    .iter()
                    .any(|clip| demand.visual_controls.contains(&clip.id));
            let audio_track = active
                && !track.state.muted
                && matches!(track.kind, TrackKind::Video | TrackKind::Audio);
            let output_audio = audio_track && demand.output_audio(track);
            let control_audio = audio_track
                && track.clips.iter().filter(|clip| clip.enabled).any(|clip| {
                    activity.audio_live(track, clip)
                        && !demand
                            .control_windows(&track.id, clip.record_range)
                            .is_empty()
                });
            let audio = output_audio || control_audio;
            let sidecar = active
                .then(|| demand.sidecars.get(&track.id).copied())
                .flatten();
            let visual = main_visual || burn_caption || control_visual;
            let state = EffectiveTrackState {
                include_in_render: visual || audio || sidecar.is_some(),
                visual_enabled: visual,
                audio_enabled: audio,
            };
            self.tracks.insert((id.clone(), track.id.clone()), state);
            for clip in track.clips.iter().filter(|clip| clip.enabled) {
                let clip_has_audio = activity.audio_live(track, clip);
                let control_windows = if audio_track {
                    demand.control_windows(&track.id, clip.record_range)
                } else {
                    Vec::new()
                };
                let audio_control_ranges = if clip_has_audio {
                    control_windows
                } else {
                    Vec::new()
                };
                self.clip(
                    sequence,
                    clip,
                    main_visual || burn_caption,
                    output_audio && clip_has_audio,
                    audio_control_ranges,
                    sidecar,
                );
            }
        }
        let mut expanded = demand;
        let clips = |item: &veac_ir::ItemId| self.clips.get(&(id.clone(), item.clone())).copied();
        if super::dependencies::close(sequence, &self.relations, clips, &mut expanded) {
            self.demands.insert(id.clone(), expanded);
            self.pending.insert(id.clone());
        }
    }

    fn clip(
        &mut self,
        sequence: &veac_ir::Sequence,
        clip: &Clip,
        main_visual: bool,
        audio_output: bool,
        audio_control_ranges: Vec<veac_ir::TimeRange>,
        sidecar: Option<TextDemand>,
    ) {
        let control = self
            .demands
            .get(&sequence.id)
            .is_some_and(|value| value.visual_controls.contains(&clip.id));
        let visual = main_visual || control;
        let mut text = sidecar.unwrap_or_default();
        if visual
            && matches!(
                clip.source,
                ClipSource::Text { .. } | ClipSource::Caption { .. }
            )
        {
            text.merge(TextDemand::Styled);
        }
        let audio_control = !audio_control_ranges.is_empty();
        if !visual && !audio_output && !audio_control && text == TextDemand::None {
            return;
        }
        let value = ClipDemand {
            visual,
            audio_output,
            audio_control,
            text,
        };
        self.clips
            .entry((sequence.id.clone(), clip.id.clone()))
            .or_default()
            .merge(value);
        if let ClipSource::Sequence { sequence_id } = &clip.source {
            let child_duration =
                super::source_window::sequence_duration(self.envelope, sequence_id);
            let child_controls =
                super::source_window::project(clip, &audio_control_ranges, child_duration);
            let child = SequenceDemand::nested(
                visual,
                audio_output,
                child_controls,
                self.demands
                    .get(&sequence.id)
                    .is_some_and(|value| value.burn_captions),
            );
            let changed = self
                .demands
                .entry(sequence_id.clone())
                .or_default()
                .merge(&child);
            if changed {
                self.pending.insert(sequence_id.clone());
            }
        }
    }
}
