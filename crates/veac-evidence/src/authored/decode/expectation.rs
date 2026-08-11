use veac_lang::program::expression::Value;

use crate::{AlphaExpectation, BoundsExpectation, DiffExpectation, RangeExpectation};

use super::super::EvidenceDecodeError;
use super::value::Decoder;

pub(super) fn range(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<RangeExpectation, EvidenceDecodeError> {
    let fields = decoder.structure(value, "RangeExpectation", path)?;
    Ok(RangeExpectation {
        minimum: decoder.optional_scalar(fields.get("minimum")?, &fields.path("minimum"))?,
        maximum: decoder.optional_scalar(fields.get("maximum")?, &fields.path("maximum"))?,
    })
}

pub(super) fn alpha(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<AlphaExpectation, EvidenceDecodeError> {
    let fields = decoder.structure(value, "AlphaExpectation", path)?;
    Ok(AlphaExpectation {
        transparent_below: decoder.u8(
            fields.get("transparent_below")?,
            &fields.path("transparent_below"),
        )?,
        opaque_above: decoder.u8(fields.get("opaque_above")?, &fields.path("opaque_above"))?,
        mean: decoder.optional_range(fields.get("mean")?, &fields.path("mean"))?,
        transparent_fraction: decoder.optional_range(
            fields.get("transparent_fraction")?,
            &fields.path("transparent_fraction"),
        )?,
        partial_fraction: decoder.optional_range(
            fields.get("partial_fraction")?,
            &fields.path("partial_fraction"),
        )?,
        opaque_fraction: decoder.optional_range(
            fields.get("opaque_fraction")?,
            &fields.path("opaque_fraction"),
        )?,
    })
}

pub(super) fn diff(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<DiffExpectation, EvidenceDecodeError> {
    let fields = decoder.structure(value, "DiffExpectation", path)?;
    Ok(DiffExpectation {
        rmse: decoder.optional_range(fields.get("rmse")?, &fields.path("rmse"))?,
        mae: decoder.optional_range(fields.get("mae")?, &fields.path("mae"))?,
        maximum_delta: decoder
            .optional_range(fields.get("maximum_delta")?, &fields.path("maximum_delta"))?,
        changed_fraction: decoder.optional_range(
            fields.get("changed_fraction")?,
            &fields.path("changed_fraction"),
        )?,
    })
}

pub(super) fn bounds(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<BoundsExpectation, EvidenceDecodeError> {
    let fields = decoder.structure(value, "BoundsExpectation", path)?;
    Ok(BoundsExpectation {
        non_empty: decoder.bool(fields.get("non_empty")?, &fields.path("non_empty"))?,
        minimum_margin_pixels: decoder.optional_u32(
            fields.get("minimum_margin_pixels")?,
            &fields.path("minimum_margin_pixels"),
        )?,
        maximum_width_pixels: decoder.optional_u32(
            fields.get("maximum_width_pixels")?,
            &fields.path("maximum_width_pixels"),
        )?,
        maximum_height_pixels: decoder.optional_u32(
            fields.get("maximum_height_pixels")?,
            &fields.path("maximum_height_pixels"),
        )?,
    })
}
