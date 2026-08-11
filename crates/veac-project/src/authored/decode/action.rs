use veac_lang::program::expression::Value;

use crate::{ProjectPath, ProjectTargetEntry};

use super::super::ProjectDecodeError;
use super::value::{unknown_variant, Decoder};

pub(super) fn entry(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProjectTargetEntry, ProjectDecodeError> {
    let variant = decoder.variant(value, "ProjectTargetEntry", path)?;
    match variant.name {
        "Veac" => Ok(ProjectTargetEntry::Veac {
            source: ProjectPath::new(decoder.text(
                variant.fields.get("source")?,
                &variant.fields.path("source"),
            )?),
        }),
        "MediaDerivation" => Ok(ProjectTargetEntry::MediaDerivation {
            operation: super::derivation::decode(
                decoder,
                variant.fields.get("operation")?,
                &variant.fields.path("operation"),
            )?,
        }),
        "Evidence" => Ok(ProjectTargetEntry::Evidence {
            contract: ProjectPath::new(decoder.text(
                variant.fields.get("contract")?,
                &variant.fields.path("contract"),
            )?),
        }),
        name => Err(unknown_variant(path, "ProjectTargetEntry", name)),
    }
}
