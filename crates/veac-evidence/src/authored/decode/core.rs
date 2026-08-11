use veac_lang::program::expression::Value;

use crate::{RegionSpace, RegionSpec, SampleSpec, SourceBinding, SourceSpec};

use super::super::EvidenceDecodeError;
use super::value::{unknown_variant, Decoder};

pub(super) fn source(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<SourceSpec, EvidenceDecodeError> {
    let fields = decoder.structure(value, "EvidenceSource", path)?;
    Ok(SourceSpec {
        id: decoder.identifier(fields.get("id")?, &fields.path("id"))?,
        binding: binding(decoder, fields.get("binding")?, &fields.path("binding"))?,
    })
}

fn binding(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<SourceBinding, EvidenceDecodeError> {
    let variant = decoder.variant(value, "EvidenceSourceBinding", path)?;
    let id = |field| decoder.identifier(variant.fields.get(field)?, &variant.fields.path(field));
    match variant.name {
        "Deliverable" => Ok(SourceBinding::Deliverable {
            deliverable_id: id("deliverable_id")?,
        }),
        "Artifact" => Ok(SourceBinding::Artifact {
            artifact_id: id("artifact_id")?,
        }),
        "BoundInput" => Ok(SourceBinding::BoundInput {
            input_id: id("input_id")?,
        }),
        name => Err(unknown_variant(path, "EvidenceSourceBinding", name)),
    }
}

pub(super) fn sample(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<SampleSpec, EvidenceDecodeError> {
    let fields = decoder.structure(value, "EvidenceSample", path)?;
    Ok(SampleSpec {
        id: decoder.identifier(fields.get("id")?, &fields.path("id"))?,
        source_id: decoder.identifier(fields.get("source_id")?, &fields.path("source_id"))?,
        at: decoder.time(fields.get("at")?, &fields.path("at"))?,
    })
}

pub(super) fn region(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<RegionSpec, EvidenceDecodeError> {
    let fields = decoder.structure(value, "EvidenceRegion", path)?;
    Ok(RegionSpec {
        id: decoder.identifier(fields.get("id")?, &fields.path("id"))?,
        space: space(decoder, fields.get("space")?, &fields.path("space"))?,
    })
}

fn space(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<RegionSpace, EvidenceDecodeError> {
    let variant = decoder.variant(value, "EvidenceRegionSpace", path)?;
    let get = |name| variant.fields.get(name);
    let at = |name| variant.fields.path(name);
    match variant.name {
        "Normalized" => Ok(RegionSpace::Normalized {
            x: decoder.scalar(get("x")?, &at("x"))?,
            y: decoder.scalar(get("y")?, &at("y"))?,
            width: decoder.scalar(get("width")?, &at("width"))?,
            height: decoder.scalar(get("height")?, &at("height"))?,
        }),
        "Pixels" => Ok(RegionSpace::Pixels {
            x: decoder.u32(get("x")?, &at("x"))?,
            y: decoder.u32(get("y")?, &at("y"))?,
            width: decoder.u32(get("width")?, &at("width"))?,
            height: decoder.u32(get("height")?, &at("height"))?,
        }),
        name => Err(unknown_variant(path, "EvidenceRegionSpace", name)),
    }
}
