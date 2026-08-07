use std::collections::BTreeSet;

use crate::temporal::{
    TemporalProgram, TemporalProgramLibrary, MAX_TEMPORAL_BINDINGS, MAX_TEMPORAL_PROGRAMS,
    MAX_TEMPORAL_PROVENANCE,
};

use super::Validator;

impl Validator {
    pub(super) fn library_limits(&mut self, library: &TemporalProgramLibrary) {
        let limits = [
            (
                library.programs.len(),
                MAX_TEMPORAL_PROGRAMS,
                "TEMPORAL_PROGRAM_LIMIT",
                "/programs",
                "program",
            ),
            (
                library.bindings.len(),
                MAX_TEMPORAL_BINDINGS,
                "TEMPORAL_BINDING_LIMIT",
                "/bindings",
                "binding",
            ),
            (
                library.provenance.len(),
                MAX_TEMPORAL_PROVENANCE,
                "TEMPORAL_PROVENANCE_LIMIT",
                "/provenance",
                "provenance",
            ),
        ];
        for (actual, limit, code, pointer, resource) in limits {
            if actual > limit {
                self.push(code, pointer, format!("temporal {resource} limit exceeded"));
            }
        }
    }

    pub(super) fn program_provenance(
        &mut self,
        program: &TemporalProgram,
        values: &BTreeSet<String>,
        pointer: &str,
    ) {
        if !values.contains(program.provenance_id.as_str()) {
            self.push(
                "TEMPORAL_PROVENANCE_MISSING",
                format!("{pointer}/provenance_id"),
                "program provenance does not exist",
            );
        }
        for (index, node) in program.nodes.iter().enumerate() {
            if node
                .provenance_id
                .as_ref()
                .is_some_and(|id| !values.contains(id.as_str()))
            {
                self.push(
                    "TEMPORAL_PROVENANCE_MISSING",
                    format!("{pointer}/nodes/{index}/provenance_id"),
                    "node provenance does not exist",
                );
            }
        }
    }
}
