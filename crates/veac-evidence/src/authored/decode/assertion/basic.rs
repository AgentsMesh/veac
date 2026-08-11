use crate::{
    AlphaSpec, AssertionSpec, BoundsSpec, DecodeCompleteSpec, LayerOrderSpec, PixelDiffSpec,
};

use super::super::super::EvidenceDecodeError;
use super::super::expectation;
use super::super::value::{Decoder, Fields};

pub(super) fn decode_complete(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
) -> Result<AssertionSpec, EvidenceDecodeError> {
    Ok(AssertionSpec::DecodeComplete(DecodeCompleteSpec {
        id: identifier(decoder, fields, "id")?,
        source_id: identifier(decoder, fields, "source_id")?,
        minimum_frames: decoder.u64(
            fields.get("minimum_frames")?,
            &fields.path("minimum_frames"),
        )?,
    }))
}

pub(super) fn alpha(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
) -> Result<AssertionSpec, EvidenceDecodeError> {
    Ok(AssertionSpec::Alpha(AlphaSpec {
        id: identifier(decoder, fields, "id")?,
        sample_id: identifier(decoder, fields, "sample_id")?,
        region_id: decoder
            .optional_identifier(fields.get("region_id")?, &fields.path("region_id"))?,
        expectation: expectation::alpha(
            decoder,
            fields.get("expectation")?,
            &fields.path("expectation"),
        )?,
    }))
}

pub(super) fn pixel_diff(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
) -> Result<AssertionSpec, EvidenceDecodeError> {
    Ok(AssertionSpec::PixelDiff(PixelDiffSpec {
        id: identifier(decoder, fields, "id")?,
        left_sample_id: identifier(decoder, fields, "left_sample_id")?,
        right_sample_id: identifier(decoder, fields, "right_sample_id")?,
        region_id: decoder
            .optional_identifier(fields.get("region_id")?, &fields.path("region_id"))?,
        channels: super::types::channels(
            decoder,
            fields.get("channels")?,
            &fields.path("channels"),
        )?,
        change_threshold: decoder.u8(
            fields.get("change_threshold")?,
            &fields.path("change_threshold"),
        )?,
        expectation: expectation::diff(
            decoder,
            fields.get("expectation")?,
            &fields.path("expectation"),
        )?,
    }))
}

pub(super) fn bounds(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
) -> Result<AssertionSpec, EvidenceDecodeError> {
    Ok(AssertionSpec::Bounds(BoundsSpec {
        id: identifier(decoder, fields, "id")?,
        sample_id: identifier(decoder, fields, "sample_id")?,
        region_id: decoder
            .optional_identifier(fields.get("region_id")?, &fields.path("region_id"))?,
        mask: super::types::mask(decoder, fields.get("mask")?, &fields.path("mask"))?,
        expectation: expectation::bounds(
            decoder,
            fields.get("expectation")?,
            &fields.path("expectation"),
        )?,
    }))
}

pub(super) fn layer_order(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
) -> Result<AssertionSpec, EvidenceDecodeError> {
    Ok(AssertionSpec::LayerOrder(LayerOrderSpec {
        id: identifier(decoder, fields, "id")?,
        upper_entity: decoder.text(fields.get("upper_entity")?, &fields.path("upper_entity"))?,
        lower_entity: decoder.text(fields.get("lower_entity")?, &fields.path("lower_entity"))?,
    }))
}

pub(super) fn identifier(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
    name: &str,
) -> Result<String, EvidenceDecodeError> {
    decoder.identifier(fields.get(name)?, &fields.path(name))
}
