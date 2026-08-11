use crate::{validate, EvidenceSuiteV1, ValidatedSuite, ValidationError};

#[derive(Debug)]
pub enum EvidenceContractError {
    Json(serde_json::Error),
    Validation(ValidationError),
}

pub fn decode_evidence_suite_json(input: &str) -> Result<ValidatedSuite, EvidenceContractError> {
    veac_ir::reject_duplicate_json_keys(input).map_err(EvidenceContractError::Json)?;
    let suite =
        serde_json::from_str::<EvidenceSuiteV1>(input).map_err(EvidenceContractError::Json)?;
    validate(suite).map_err(EvidenceContractError::Validation)
}

impl std::fmt::Display for EvidenceContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "evidence JSON is invalid: {error}"),
            Self::Validation(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for EvidenceContractError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::Validation(error) => Some(error),
        }
    }
}
