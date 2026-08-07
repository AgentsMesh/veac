mod binding;
mod composite;
mod curve;
mod library;
mod node;
mod operation;
mod program;
mod provenance;
mod value;

use super::{
    TemporalDiagnostic, TemporalProgram, TemporalProgramLibrary, TemporalValidationErrors,
    MAX_TEMPORAL_NODES, MAX_TEMPORAL_VALUE_BYTES,
};

#[derive(Default)]
struct Validator {
    diagnostics: Vec<TemporalDiagnostic>,
    nodes: usize,
    value_bytes: usize,
}

impl Validator {
    fn push(&mut self, code: &str, pointer: impl Into<String>, message: impl Into<String>) {
        self.diagnostics.push(TemporalDiagnostic {
            code: code.to_owned(),
            pointer: pointer.into(),
            message: message.into(),
        });
    }

    fn finish(self) -> Result<(), TemporalValidationErrors> {
        if self.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(TemporalValidationErrors::new(self.diagnostics))
        }
    }

    fn charge_nodes(&mut self, amount: usize, pointer: &str) {
        self.nodes = self.nodes.saturating_add(amount);
        if self.nodes > MAX_TEMPORAL_NODES {
            self.push(
                "TEMPORAL_NODE_LIMIT",
                pointer,
                "temporal node limit exceeded",
            );
        }
    }

    fn charge_value(&mut self, amount: usize, pointer: &str) {
        self.value_bytes = self.value_bytes.saturating_add(amount);
        if self.value_bytes > MAX_TEMPORAL_VALUE_BYTES {
            self.push(
                "TEMPORAL_VALUE_LIMIT",
                pointer,
                "temporal value byte limit exceeded",
            );
        }
    }
}

pub fn validate_temporal_program(
    program: &TemporalProgram,
) -> Result<(), TemporalValidationErrors> {
    let mut validator = Validator::default();
    validator.program(program, "/program");
    validator.finish()
}

pub fn validate_temporal_library(
    library: &TemporalProgramLibrary,
) -> Result<(), TemporalValidationErrors> {
    let mut validator = Validator::default();
    validator.library(library);
    validator.finish()
}
