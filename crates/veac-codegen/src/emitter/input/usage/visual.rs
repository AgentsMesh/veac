use std::collections::BTreeSet;

use veac_artifact::MediaRole;
use veac_plan::canonical::{ItemId, SequenceId, TrackKind};
use veac_plan::{
    ResolvedApply, ResolvedApplyOperation, ResolvedClip, ResolvedClipSource, ResolvedColorStage,
    ResolvedRenderPlan, ResolvedSequence,
};

use super::{text_resources, Usage};

pub(super) fn collect(plan: &ResolvedRenderPlan, captions: bool, usage: &mut Usage) {
    Walker {
        plan,
        captions,
        sequences: BTreeSet::new(),
        clips: BTreeSet::new(),
        usage,
    }
    .sequence(&plan.entry_sequence_id);
}

struct Walker<'a> {
    plan: &'a ResolvedRenderPlan,
    captions: bool,
    sequences: BTreeSet<SequenceId>,
    clips: BTreeSet<(SequenceId, ItemId)>,
    usage: &'a mut Usage,
}

impl Walker<'_> {
    fn sequence(&mut self, id: &SequenceId) {
        if !self.sequences.insert(id.clone()) {
            return;
        }
        let Some(sequence) = self.plan.sequences.iter().find(|value| value.id == *id) else {
            return;
        };
        for track in &sequence.tracks {
            let visible = track.state.visual_enabled
                && matches!(
                    track.kind,
                    TrackKind::Video | TrackKind::Visual | TrackKind::Caption
                )
                && (track.kind != TrackKind::Caption || self.captions);
            if !visible {
                continue;
            }
            for clip in &track.clips {
                if clip.visual.is_some() {
                    self.clip(sequence, clip);
                }
            }
        }
        for apply in &sequence.applies {
            self.apply(apply);
        }
    }

    fn apply(&mut self, apply: &ResolvedApply) {
        for stage in apply
            .stages
            .iter()
            .filter(|stage| crate::emitter::apply::stage_used(apply, stage))
        {
            let ResolvedApplyOperation::Color { pipeline } = &stage.operation else {
                continue;
            };
            for color_stage in &pipeline.stages {
                if let ResolvedColorStage::Lut { application } = color_stage {
                    self.usage.resource(&application.input_id);
                }
            }
        }
    }

    fn clip(&mut self, sequence: &ResolvedSequence, clip: &ResolvedClip) {
        if !self.clips.insert((sequence.id.clone(), clip.id.clone())) {
            return;
        }
        if let Some(visual) = &clip.visual {
            if let Some(pipeline) = &visual.color_pipeline {
                for stage in &pipeline.stages {
                    if let ResolvedColorStage::Lut { application } = stage {
                        self.usage.resource(&application.input_id);
                    }
                }
            }
            if let Some(matte) = &visual.track_matte {
                if let Some(source) = sequence
                    .tracks
                    .iter()
                    .flat_map(|track| &track.clips)
                    .find(|value| value.id == matte.source_clip_id)
                {
                    self.clip(sequence, source);
                }
            }
        }
        match &clip.source {
            ResolvedClipSource::Media {
                input_id,
                video_stream: Some(_),
                ..
            }
            | ResolvedClipSource::FreezeFrame { input_id, .. } => {
                self.usage.media(input_id, MediaRole::Video);
            }
            ResolvedClipSource::Sequence { sequence_id } => self.sequence(sequence_id),
            ResolvedClipSource::Multicam { source } => {
                for switch in &source.switches {
                    if let Some(angle) = source
                        .angles
                        .iter()
                        .find(|angle| angle.id == switch.angle_id)
                    {
                        self.usage.media(&angle.input_id, MediaRole::Video);
                    }
                }
            }
            ResolvedClipSource::Text { content } | ResolvedClipSource::Caption { content, .. } => {
                text_resources(content, true, self.usage);
            }
            _ => {}
        }
    }
}
