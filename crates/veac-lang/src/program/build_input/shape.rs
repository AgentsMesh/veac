use super::{BuildInputManifestValue, BuildInputsError};
use crate::program::expression::MAX_TEXT_VALUE_BYTES;

impl BuildInputManifestValue {
    pub(crate) fn validate_shape(&self) -> Result<(), BuildInputsError> {
        match self {
            Self::Text { value } if value.len() > MAX_TEXT_VALUE_BYTES => {
                Err(BuildInputsError::new(
                    "PROGRAM_INPUT_VALUE",
                    format!("text Build input exceeds the {MAX_TEXT_VALUE_BYTES} byte limit"),
                ))
            }
            Self::Enum { value } if !crate::name::is_name(value) => Err(BuildInputsError::new(
                "PROGRAM_INPUT_VALUE",
                "enum Build input value must be a VEAC declaration name",
            )),
            Self::Material { .. } => super::material::validate(self),
            _ => Ok(()),
        }
    }
}
