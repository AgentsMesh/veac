use veac_lang::program::expression::Value;

use crate::{
    AssertionSpec, CompositeOverSpec, MotionProfileSpec, RevealCheckpoint, RevealOrderSpec,
    RevealSubject,
};

use super::super::super::EvidenceDecodeError;
use super::super::value::{Decoder, Fields};

pub(super) fn composite(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
) -> Result<AssertionSpec, EvidenceDecodeError> {
    Ok(AssertionSpec::CompositeOver(CompositeOverSpec {
        id: id(decoder, fields, "id")?,
        actual_sample_id: id(decoder, fields, "actual_sample_id")?,
        underlay_sample_id: id(decoder, fields, "underlay_sample_id")?,
        overlay_sample_id: id(decoder, fields, "overlay_sample_id")?,
        region_id: decoder
            .optional_identifier(fields.get("region_id")?, &fields.path("region_id"))?,
        overlay_alpha_minimum: decoder.u8(
            fields.get("overlay_alpha_minimum")?,
            &fields.path("overlay_alpha_minimum"),
        )?,
        maximum_expected_rmse: decoder.scalar(
            fields.get("maximum_expected_rmse")?,
            &fields.path("maximum_expected_rmse"),
        )?,
        minimum_improvement_ratio: decoder.scalar(
            fields.get("minimum_improvement_ratio")?,
            &fields.path("minimum_improvement_ratio"),
        )?,
    }))
}

pub(super) fn reveal(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
) -> Result<AssertionSpec, EvidenceDecodeError> {
    Ok(AssertionSpec::RevealOrder(RevealOrderSpec {
        id: id(decoder, fields, "id")?,
        baseline_sample_id: id(decoder, fields, "baseline_sample_id")?,
        subjects: decoder.list_map(fields.get("subjects")?, &fields.path("subjects"), subject)?,
        checkpoints: decoder.list_map(
            fields.get("checkpoints")?,
            &fields.path("checkpoints"),
            checkpoint,
        )?,
        change_threshold: decoder.u8(
            fields.get("change_threshold")?,
            &fields.path("change_threshold"),
        )?,
        visible_minimum: decoder.scalar(
            fields.get("visible_minimum")?,
            &fields.path("visible_minimum"),
        )?,
        hidden_maximum: decoder.scalar(
            fields.get("hidden_maximum")?,
            &fields.path("hidden_maximum"),
        )?,
    }))
}

pub(super) fn motion(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
) -> Result<AssertionSpec, EvidenceDecodeError> {
    Ok(AssertionSpec::MotionProfile(MotionProfileSpec {
        id: id(decoder, fields, "id")?,
        sample_ids: decoder.list_map(
            fields.get("sample_ids")?,
            &fields.path("sample_ids"),
            identifier_value,
        )?,
        region_id: decoder
            .optional_identifier(fields.get("region_id")?, &fields.path("region_id"))?,
        metric: super::types::motion_metric(
            decoder,
            fields.get("metric")?,
            &fields.path("metric"),
        )?,
        minimum_interval_motion: decoder.scalar(
            fields.get("minimum_interval_motion")?,
            &fields.path("minimum_interval_motion"),
        )?,
        minimum_deceleration_ratio: decoder.scalar(
            fields.get("minimum_deceleration_ratio")?,
            &fields.path("minimum_deceleration_ratio"),
        )?,
        monotonic_tolerance: decoder.scalar(
            fields.get("monotonic_tolerance")?,
            &fields.path("monotonic_tolerance"),
        )?,
    }))
}

fn subject(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<RevealSubject, EvidenceDecodeError> {
    let fields = decoder.structure(value, "RevealSubject", path)?;
    Ok(RevealSubject {
        id: id(decoder, &fields, "id")?,
        region_id: id(decoder, &fields, "region_id")?,
    })
}

fn checkpoint(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<RevealCheckpoint, EvidenceDecodeError> {
    let fields = decoder.structure(value, "RevealCheckpoint", path)?;
    Ok(RevealCheckpoint {
        sample_id: id(decoder, &fields, "sample_id")?,
        visible_prefix: decoder.usize(
            fields.get("visible_prefix")?,
            &fields.path("visible_prefix"),
        )?,
    })
}

fn identifier_value(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<String, EvidenceDecodeError> {
    decoder.identifier(value, path)
}

fn id(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
    name: &str,
) -> Result<String, EvidenceDecodeError> {
    decoder.identifier(fields.get(name)?, &fields.path(name))
}
