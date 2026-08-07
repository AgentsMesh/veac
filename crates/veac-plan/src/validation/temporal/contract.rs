use std::collections::{BTreeMap, BTreeSet};

use veac_ir::TemporalClockOwner;

use crate::{temporal::TemporalSink, ResolvedRenderPlan};

use super::super::Validator;

pub(super) struct Contract<'a, 'b> {
    pub(super) validator: &'a mut Validator,
    pub(super) plan: &'b ResolvedRenderPlan,
    bindings: BTreeMap<&'b str, usize>,
    pub(super) used_bindings: BTreeSet<String>,
    sequences: BTreeSet<&'b str>,
    items: BTreeSet<&'b str>,
}

impl<'a, 'b> Contract<'a, 'b> {
    pub(super) fn new(validator: &'a mut Validator, plan: &'b ResolvedRenderPlan) -> Self {
        let bindings = plan
            .temporal
            .bindings
            .iter()
            .enumerate()
            .map(|(index, value)| (value.id.as_str(), index))
            .collect();
        let sequences = plan
            .sequences
            .iter()
            .map(|value| value.id.as_str())
            .collect();
        let items = plan
            .sequences
            .iter()
            .flat_map(|sequence| &sequence.tracks)
            .flat_map(|track| &track.clips)
            .map(|clip| clip.id.as_str())
            .collect();
        Self {
            validator,
            plan,
            bindings,
            used_bindings: BTreeSet::new(),
            sequences,
            items,
        }
    }

    pub(super) fn validate(mut self) {
        self.clock_owners();
        for sink in crate::temporal::collect(&self.plan.sequences) {
            self.sink(sink);
        }
        self.reachability();
    }

    fn clock_owners(&mut self) {
        for binding_index in 0..self.plan.temporal.bindings.len() {
            for clock_index in 0..self.plan.temporal.bindings[binding_index].clocks.len() {
                let owner = self.plan.temporal.bindings[binding_index].clocks[clock_index]
                    .owner
                    .clone();
                let exists = match &owner {
                    TemporalClockOwner::Sequence { sequence_id } => {
                        self.sequences.contains(sequence_id.as_str())
                    }
                    TemporalClockOwner::Item { item_id } => self.items.contains(item_id.as_str()),
                };
                if !exists {
                    self.validator.push(
                        "PLAN_TEMPORAL_CLOCK_OWNER_MISSING",
                        format!("/temporal/bindings/{binding_index}/clocks/{clock_index}/owner"),
                        "temporal clock owner is outside the resolved plan graph",
                    );
                }
            }
        }
    }

    fn sink(&mut self, sink: TemporalSink) {
        self.used_bindings.insert(sink.binding_id.to_string());
        let Some(index) = self.bindings.get(sink.binding_id.as_str()).copied() else {
            self.validator.push(
                "PLAN_TEMPORAL_BINDING_MISSING",
                &sink.pointer,
                "animation sink references an unknown temporal binding",
            );
            return;
        };
        let binding = &self.plan.temporal.bindings[index];
        if binding.result_type != sink.expected {
            self.validator.push(
                "PLAN_TEMPORAL_SINK_TYPE",
                &sink.pointer,
                "animation sink type does not match its temporal binding",
            );
        }
        if let Some(program) = self
            .plan
            .temporal
            .programs
            .iter()
            .find(|program| program.id == binding.program_id)
        {
            if program.result_type != sink.expected {
                self.validator.push(
                    "PLAN_TEMPORAL_PROGRAM_TYPE",
                    &sink.pointer,
                    "animation sink type does not match its temporal program",
                );
            }
        }
        for clock in &binding.clocks {
            if !owner_matches(&sink, &clock.owner) {
                self.validator.push(
                    "PLAN_TEMPORAL_CROSS_OWNER",
                    &sink.pointer,
                    "temporal clock owner does not match the animation sink owner",
                );
            }
        }
    }
}

fn owner_matches(sink: &TemporalSink, owner: &TemporalClockOwner) -> bool {
    match owner {
        TemporalClockOwner::Sequence { sequence_id } => sequence_id == &sink.scope.sequence_id,
        TemporalClockOwner::Item { item_id } => sink.scope.item_id.as_ref() == Some(item_id),
    }
}
