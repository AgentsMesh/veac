use super::super::{ResidualizationError, ResidualizationRequest};
use veac_ir::{
    validate_temporal_library, TemporalProgram, TemporalProgramLibrary, TEMPORAL_OPSET_VERSION,
};

pub(super) fn library(
    program: Option<&TemporalProgram>,
    request: &ResidualizationRequest,
) -> Result<(), ResidualizationError> {
    let library = TemporalProgramLibrary {
        opset_version: TEMPORAL_OPSET_VERSION,
        programs: program.into_iter().cloned().collect(),
        bindings: Vec::new(),
        provenance: vec![request.provenance.clone()],
    };
    validate_temporal_library(&library).map_err(|errors| {
        let value = &errors.diagnostics()[0];
        ResidualizationError::new(
            "RESIDUAL_TEMPORAL_VALIDATE",
            format!("{} at {}: {}", value.code, value.pointer, value.message),
            0..0,
        )
    })
}
