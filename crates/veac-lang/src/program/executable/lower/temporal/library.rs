use std::collections::{BTreeMap, BTreeSet};

use veac_ir::{
    validate_temporal_library, TemporalBinding, TemporalBindingId, TemporalProgram,
    TemporalProgramId, TemporalProgramLibrary, TemporalProvenance, TemporalProvenanceId,
    TEMPORAL_OPSET_VERSION,
};

use super::{error, ExecutableLowerError};

#[derive(Default)]
pub(super) struct Builder {
    programs: Vec<TemporalProgram>,
    bindings: Vec<TemporalBinding>,
    provenance: BTreeMap<TemporalProvenanceId, TemporalProvenance>,
    program_digests: BTreeMap<String, TemporalProgramId>,
    program_ids: BTreeMap<TemporalProgramId, String>,
    binding_ids: BTreeSet<TemporalBindingId>,
}

impl Builder {
    pub(super) fn program(
        &mut self,
        program: TemporalProgram,
    ) -> Result<TemporalProgramId, ExecutableLowerError> {
        if let Some(previous) = self.program_ids.get(&program.id) {
            if previous != &program.content_sha256 {
                return Err(error(
                    "EXECUTABLE_TEMPORAL_PROGRAM_ID",
                    "one Temporal program ID names different residual programs",
                ));
            }
        }
        let id = program.id.clone();
        self.program_ids
            .insert(id.clone(), program.content_sha256.clone());
        if let Some(id) = self.program_digests.get(&program.content_sha256) {
            return Ok(id.clone());
        }
        self.program_digests
            .insert(program.content_sha256.clone(), id.clone());
        self.programs.push(program);
        Ok(id)
    }

    pub(super) fn binding(&mut self, binding: TemporalBinding) -> Result<(), ExecutableLowerError> {
        if !self.binding_ids.insert(binding.id.clone()) {
            return Err(error(
                "EXECUTABLE_TEMPORAL_BINDING_ID",
                "one Temporal binding ID is assigned to more than one animation leaf",
            ));
        }
        self.bindings.push(binding);
        Ok(())
    }

    pub(super) fn provenance(
        &mut self,
        value: TemporalProvenance,
    ) -> Result<(), ExecutableLowerError> {
        if let Some(previous) = self.provenance.get(&value.id) {
            if previous != &value {
                return Err(error(
                    "EXECUTABLE_TEMPORAL_PROVENANCE_ID",
                    "one Temporal provenance ID names different authored sites",
                ));
            }
            return Ok(());
        }
        self.provenance.insert(value.id.clone(), value);
        Ok(())
    }

    pub(super) fn finish(self) -> Result<TemporalProgramLibrary, ExecutableLowerError> {
        let library = TemporalProgramLibrary {
            opset_version: TEMPORAL_OPSET_VERSION,
            programs: self.programs,
            bindings: self.bindings,
            provenance: self.provenance.into_values().collect(),
        };
        validate_temporal_library(&library).map_err(|errors| {
            let diagnostic = &errors.diagnostics()[0];
            error(
                "EXECUTABLE_TEMPORAL_LIBRARY",
                format!(
                    "{} at {}: {}",
                    diagnostic.code, diagnostic.pointer, diagnostic.message
                ),
            )
        })?;
        Ok(library)
    }
}
