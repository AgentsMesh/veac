use std::collections::BTreeMap;

use veac_plan::canonical::TrackId;
use veac_plan::{EffectiveVisualProperties, ResolvedClip, ResolvedSequence, ResolvedTrack};

use super::super::{CodegenErrors, EmitContext};

pub(super) struct Band {
    pub target_position: usize,
    pub label: String,
    pub underlay: Option<String>,
    pub remaining: usize,
    checkpoints: BTreeMap<TrackId, Checkpoint>,
}

struct Checkpoint {
    underlay: String,
    remaining: usize,
}

impl Band {
    pub fn new(
        context: &mut EmitContext<'_>,
        sequence: &ResolvedSequence,
        target_position: usize,
        remaining: usize,
    ) -> Self {
        Self {
            target_position,
            label: transparent(context, sequence),
            underlay: None,
            remaining,
            checkpoints: BTreeMap::new(),
        }
    }

    pub fn checkpoint(&mut self, context: &mut EmitContext<'_>, target: TrackId, uses: usize) {
        if self.checkpoints.contains_key(&target) {
            return;
        }
        let (underlay, continuing) = context.graph.split(&self.label, "applynestedcheckpointv");
        self.label = continuing;
        self.checkpoints.insert(
            target,
            Checkpoint {
                underlay,
                remaining: uses,
            },
        );
    }

    pub fn main_underlay(&mut self, context: &mut EmitContext<'_>) -> String {
        let underlay = self
            .underlay
            .take()
            .expect("apply target checkpoint exists");
        if self.remaining == 1 {
            return underlay;
        }
        let (used, kept) = context.graph.split(&underlay, "applyunderlaysplitv");
        self.underlay = Some(kept);
        used
    }

    pub fn nested_underlay(&mut self, context: &mut EmitContext<'_>, target: &TrackId) -> String {
        let checkpoint = self.checkpoints.get_mut(target).expect("checkpoint exists");
        let underlay = std::mem::take(&mut checkpoint.underlay);
        checkpoint.remaining -= 1;
        if checkpoint.remaining == 0 {
            self.checkpoints.remove(target);
            return underlay;
        }
        let (used, kept) = context.graph.split(&underlay, "applynestedunderlaysplitv");
        self.checkpoints
            .get_mut(target)
            .expect("checkpoint remains")
            .underlay = kept;
        used
    }

    pub fn add_layer(
        &mut self,
        context: &mut EmitContext<'_>,
        sequence: &ResolvedSequence,
        track: &ResolvedTrack,
        clip: &ResolvedClip,
        visual: &EffectiveVisualProperties,
    ) -> Result<(), CodegenErrors> {
        self.label = super::super::visual::compose_layer(
            context,
            sequence,
            std::mem::take(&mut self.label),
            track,
            clip,
            visual,
        )?;
        Ok(())
    }
}

fn transparent(context: &mut EmitContext<'_>, sequence: &ResolvedSequence) -> String {
    let canvas = context.canvas;
    context.graph.source(
        format!(
            "color=c=black@0:s={}x{}:r={}/{}:d={},format=rgba",
            canvas.width,
            canvas.height,
            canvas.frame_rate.numerator,
            canvas.frame_rate.denominator,
            super::super::time::seconds(sequence.duration)
        ),
        "applybandv",
    )
}
