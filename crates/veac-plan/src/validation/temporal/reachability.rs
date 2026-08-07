use std::collections::BTreeSet;

use super::contract::Contract;

impl Contract<'_, '_> {
    pub(super) fn reachability(&mut self) {
        let mut programs = BTreeSet::new();
        let mut provenance = BTreeSet::new();
        for index in 0..self.plan.temporal.bindings.len() {
            let binding = &self.plan.temporal.bindings[index];
            if !self.used_bindings.contains(binding.id.as_str()) {
                self.validator.push(
                    "PLAN_TEMPORAL_BINDING_ORPHAN",
                    format!("/temporal/bindings/{index}"),
                    "temporal binding is unreachable from resolved animation sinks",
                );
                continue;
            }
            programs.insert(binding.program_id.as_str());
            provenance.insert(binding.provenance_id.as_str());
        }
        for index in 0..self.plan.temporal.programs.len() {
            let program = &self.plan.temporal.programs[index];
            if !programs.contains(program.id.as_str()) {
                self.validator.push(
                    "PLAN_TEMPORAL_PROGRAM_ORPHAN",
                    format!("/temporal/programs/{index}"),
                    "temporal program is unreachable from resolved bindings",
                );
                continue;
            }
            provenance.insert(program.provenance_id.as_str());
            for node in &program.nodes {
                if let Some(id) = &node.provenance_id {
                    provenance.insert(id.as_str());
                }
            }
        }
        for index in 0..self.plan.temporal.provenance.len() {
            if !provenance.contains(self.plan.temporal.provenance[index].id.as_str()) {
                self.validator.push(
                    "PLAN_TEMPORAL_PROVENANCE_ORPHAN",
                    format!("/temporal/provenance/{index}"),
                    "temporal provenance is unreachable from resolved programs and bindings",
                );
            }
        }
    }
}
