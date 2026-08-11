use veac_lang::program::expression::Value;

use crate::{DiffChannels, MaskSpec, MotionMetric};

use super::super::super::EvidenceDecodeError;
use super::super::value::{unknown_variant, Decoder};

pub(super) fn channels(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<DiffChannels, EvidenceDecodeError> {
    let variant = decoder.variant(value, "DiffChannels", path)?;
    match variant.name {
        "Rgb" => Ok(DiffChannels::Rgb),
        "Rgba" => Ok(DiffChannels::Rgba),
        "Alpha" => Ok(DiffChannels::Alpha),
        name => Err(unknown_variant(path, "DiffChannels", name)),
    }
}

pub(super) fn mask(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<MaskSpec, EvidenceDecodeError> {
    let variant = decoder.variant(value, "MaskSpec", path)?;
    let get = |name| variant.fields.get(name);
    let at = |name| variant.fields.path(name);
    match variant.name {
        "Alpha" => Ok(MaskSpec::Alpha {
            minimum: decoder.u8(get("minimum")?, &at("minimum"))?,
        }),
        "Difference" => Ok(MaskSpec::Difference {
            reference_sample_id: decoder
                .identifier(get("reference_sample_id")?, &at("reference_sample_id"))?,
            minimum_delta: decoder.u8(get("minimum_delta")?, &at("minimum_delta"))?,
        }),
        "Luma" => Ok(MaskSpec::Luma {
            threshold: decoder.u8(get("threshold")?, &at("threshold"))?,
            above: decoder.bool(get("above")?, &at("above"))?,
        }),
        name => Err(unknown_variant(path, "MaskSpec", name)),
    }
}

pub(super) fn motion_metric(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<MotionMetric, EvidenceDecodeError> {
    let variant = decoder.variant(value, "MotionMetric", path)?;
    match variant.name {
        "ChangedFraction" => Ok(MotionMetric::ChangedFraction {
            threshold: decoder.u8(
                variant.fields.get("threshold")?,
                &variant.fields.path("threshold"),
            )?,
        }),
        "AlphaCentroid" => Ok(MotionMetric::AlphaCentroid {
            minimum: decoder.u8(
                variant.fields.get("minimum")?,
                &variant.fields.path("minimum"),
            )?,
        }),
        name => Err(unknown_variant(path, "MotionMetric", name)),
    }
}
