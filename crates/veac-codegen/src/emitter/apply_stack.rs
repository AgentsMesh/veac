mod band;
mod schedule;

use std::collections::{BTreeMap, HashMap};

use veac_plan::canonical::{ApplyId, TrackId};
use veac_plan::{
    EffectiveVisualProperties, ResolvedApply, ResolvedClip, ResolvedSequence, ResolvedTrack,
};

use super::{apply, CodegenErrors, EmitContext};
use band::Band;

pub(super) struct ApplyStack<'a> {
    bands: HashMap<TrackId, Band>,
    due: BTreeMap<usize, Vec<&'a ResolvedApply>>,
    checkpoints: HashMap<TrackId, Vec<(TrackId, usize)>>,
    propagation: HashMap<ApplyId, Vec<TrackId>>,
}

impl<'a> ApplyStack<'a> {
    pub(super) fn new(context: &mut EmitContext<'_>, sequence: &'a ResolvedSequence) -> Self {
        let schedule = schedule::build(sequence);
        let bands = schedule
            .counts
            .into_iter()
            .map(|(target, count)| {
                let position = schedule.positions[&target];
                (target, Band::new(context, sequence, position, count))
            })
            .collect();
        Self {
            bands,
            due: schedule.due,
            checkpoints: schedule.checkpoints,
            propagation: schedule.propagation,
        }
    }

    pub(super) fn checkpoint(
        &mut self,
        context: &mut EmitContext<'_>,
        position: usize,
        track: &ResolvedTrack,
        current: String,
    ) -> String {
        let Some(band) = self.bands.get(&track.id) else {
            return current;
        };
        if band.target_position != position || band.underlay.is_some() {
            return current;
        }
        let (underlay, continuing) = context.graph.split(&current, "applycheckpointv");
        self.bands.get_mut(&track.id).expect("band exists").underlay = Some(underlay);
        for (candidate, uses) in self.checkpoints.get(&track.id).cloned().unwrap_or_default() {
            self.bands
                .get_mut(&candidate)
                .expect("broader band exists")
                .checkpoint(context, track.id.clone(), uses);
        }
        continuing
    }

    pub(super) fn targets(&self, position: usize) -> Vec<TrackId> {
        self.bands
            .iter()
            .filter(|(_, band)| band.target_position <= position)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub(super) fn take_due(&mut self, position: usize) -> Vec<&'a ResolvedApply> {
        self.due.remove(&position).unwrap_or_default()
    }

    pub(super) fn add_layer(
        &mut self,
        context: &mut EmitContext<'_>,
        target: &TrackId,
        sequence: &ResolvedSequence,
        track: &ResolvedTrack,
        clip: &ResolvedClip,
        visual: &EffectiveVisualProperties,
    ) -> Result<(), CodegenErrors> {
        self.bands
            .get_mut(target)
            .expect("active apply band exists")
            .add_layer(context, sequence, track, clip, visual)
    }

    pub(super) fn apply(
        &mut self,
        context: &mut EmitContext<'_>,
        sequence: &ResolvedSequence,
        current: String,
        value: &ResolvedApply,
    ) -> Result<String, CodegenErrors> {
        let target = schedule::target_id(value).expect("scheduled apply has a band target");
        let mut band = self.bands.remove(&target).expect("apply band exists");
        let processed = apply::render(
            context,
            sequence,
            value,
            std::mem::take(&mut band.label),
            schedule::full_range(sequence),
        )?;
        let broader = self.propagation.get(&value.id).cloned().unwrap_or_default();
        let keep = band.remaining > 1;
        let outputs = 1 + usize::from(keep) + broader.len();
        let mut branches = if outputs == 1 {
            vec![processed]
        } else {
            context.graph.filter_many(
                &[&processed],
                format!("split={outputs}"),
                "applyprocessedbandsplitv",
                outputs,
            )
        };
        let processed = branches.remove(0);
        let underlay = band.main_underlay(context);
        let replacement = apply::join(context, underlay, &processed, sequence);
        let current = apply::replace(context, current, replacement, value.record_range);
        if keep {
            band.label = branches.remove(0);
            band.remaining -= 1;
            self.bands.insert(target.clone(), band);
        }
        for (candidate, processed) in broader.into_iter().zip(branches) {
            let band = self.bands.get_mut(&candidate).expect("broader band exists");
            let underlay = band.nested_underlay(context, &target);
            let replacement = apply::join(context, underlay, &processed, sequence);
            band.label = apply::replace(
                context,
                std::mem::take(&mut band.label),
                replacement,
                value.record_range,
            );
        }
        Ok(current)
    }
}
