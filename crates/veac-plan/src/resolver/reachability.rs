mod apply;
mod demand;
mod dependencies;
mod material;
mod source_window;
mod walk;

#[cfg(test)]
#[path = "reachability/source_window_tests.rs"]
mod source_window_tests;

use std::collections::{BTreeMap, BTreeSet};

use veac_ir::{ItemId, MaterialId, ProjectEnvelope, RenderConfig, SequenceId, TrackId};

use crate::EffectiveTrackState;

pub(super) use demand::TextDemand;
use material::MaterialUse;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct ClipDemand {
    pub visual: bool,
    pub audio_output: bool,
    pub audio_control: bool,
    pub text: TextDemand,
}

impl ClipDemand {
    fn merge(&mut self, other: Self) {
        self.visual |= other.visual;
        self.audio_output |= other.audio_output;
        self.audio_control |= other.audio_control;
        self.text.merge(other.text);
    }

    pub(super) fn audio(self) -> bool {
        self.audio_output || self.audio_control
    }
}

pub(super) struct Reachability {
    track_states: BTreeMap<(SequenceId, TrackId), EffectiveTrackState>,
    clip_demands: BTreeMap<(SequenceId, ItemId), ClipDemand>,
    materials: BTreeMap<MaterialId, MaterialUse>,
}

impl Reachability {
    pub(super) fn analyze(envelope: &ProjectEnvelope, config: &RenderConfig) -> Self {
        let (track_states, clip_demands) = walk::analyze(envelope, config);
        let materials = material::collect(envelope, &track_states, &clip_demands);
        Self {
            track_states,
            clip_demands,
            materials,
        }
    }

    pub(super) fn track_state(
        &self,
        sequence_id: &SequenceId,
        track_id: &TrackId,
    ) -> EffectiveTrackState {
        self.track_states
            .get(&(sequence_id.clone(), track_id.clone()))
            .copied()
            .unwrap_or(EffectiveTrackState {
                include_in_render: false,
                visual_enabled: false,
                audio_enabled: false,
            })
    }

    pub(super) fn clip(&self, sequence_id: &SequenceId, item_id: &ItemId) -> Option<ClipDemand> {
        self.clip_demands
            .get(&(sequence_id.clone(), item_id.clone()))
            .copied()
    }

    pub(super) fn is_audio_control(&self, sequence_id: &SequenceId, item_id: &ItemId) -> bool {
        self.clip(sequence_id, item_id)
            .is_some_and(|demand| demand.audio_control)
    }

    pub(super) fn material_ids(&self) -> BTreeSet<MaterialId> {
        self.materials.keys().cloned().collect()
    }
}
