mod assertion;
mod core;
mod expectation;
mod value;

use veac_lang::program::expression::Value;
use veac_lang::program::TypeRegistry;

use crate::EvidenceSuiteV1;

use self::value::Decoder;
use super::EvidenceDecodeError;

pub(super) fn suite(
    value: &Value,
    types: &TypeRegistry,
) -> Result<EvidenceSuiteV1, EvidenceDecodeError> {
    let decoder = Decoder::new(types);
    let fields = decoder.structure(value, "EvidenceSuite", "suite")?;
    Ok(EvidenceSuiteV1 {
        schema_version: decoder.u32(
            fields.get("schema_version")?,
            &fields.path("schema_version"),
        )?,
        id: decoder.identifier(fields.get("id")?, &fields.path("id"))?,
        sources: decoder.list_map(
            fields.get("sources")?,
            &fields.path("sources"),
            core::source,
        )?,
        samples: decoder.list_map(
            fields.get("samples")?,
            &fields.path("samples"),
            core::sample,
        )?,
        regions: decoder.list_map(
            fields.get("regions")?,
            &fields.path("regions"),
            core::region,
        )?,
        assertions: decoder.list_map(
            fields.get("assertions")?,
            &fields.path("assertions"),
            assertion::assertion,
        )?,
    })
}

#[cfg(test)]
mod tests;
