use std::collections::BTreeSet;

use super::TemporalContract;

impl TemporalContract<'_, '_> {
    pub(super) fn reachability(&mut self) {
        let mut programs = BTreeSet::new();
        let mut provenance = BTreeSet::new();
        for binding in &self.library.bindings {
            if !self.used_bindings.contains(binding.id.as_str()) {
                self.push(
                    "TEMPORAL_BINDING_ORPHAN",
                    "/temporal/bindings",
                    "temporal binding is not reachable from an animation sink",
                );
                continue;
            }
            programs.insert(binding.program_id.as_str());
            provenance.insert(binding.provenance_id.as_str());
        }
        for program in &self.library.programs {
            if !programs.contains(program.id.as_str()) {
                self.push(
                    "TEMPORAL_PROGRAM_ORPHAN",
                    "/temporal/programs",
                    "temporal program is not reachable from a binding",
                );
                continue;
            }
            provenance.insert(program.provenance_id.as_str());
            provenance.extend(
                program
                    .nodes
                    .iter()
                    .filter_map(|node| node.provenance_id.as_ref())
                    .map(|id| id.as_str()),
            );
        }
        for value in &self.library.provenance {
            if !provenance.contains(value.id.as_str()) {
                self.push(
                    "TEMPORAL_PROVENANCE_ORPHAN",
                    "/temporal/provenance",
                    "temporal provenance is not reachable from a program or binding",
                );
            }
        }
    }
}
