use veac_lang::program::expression::Value;

use crate::{AxisId, AxisValue, MatrixAxis, ProfileId, ProjectTarget, TargetId};

use super::super::ProjectDecodeError;
use super::action::entry;
use super::input::input;
use super::output::{delivery, output};
use super::selector::target_ref;
use super::value::Decoder;

pub(super) fn target(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProjectTarget, ProjectDecodeError> {
    let fields = decoder.structure(value, "ProjectTarget", path)?;
    Ok(ProjectTarget {
        id: TargetId::new(decoder.identifier(fields.get("id")?, &fields.path("id"))?),
        entry: entry(decoder, fields.get("entry")?, &fields.path("entry"))?,
        localized: decoder.bool(fields.get("localized")?, &fields.path("localized"))?,
        inputs: decoder.list_map(fields.get("inputs")?, &fields.path("inputs"), input)?,
        profiles: decoder.list_map(
            fields.get("profiles")?,
            &fields.path("profiles"),
            profile_id,
        )?,
        axes: decoder.list_map(fields.get("axes")?, &fields.path("axes"), axis)?,
        needs: decoder.list_map(fields.get("needs")?, &fields.path("needs"), target_ref)?,
        outputs: decoder.list_map(fields.get("outputs")?, &fields.path("outputs"), output)?,
        deliveries: decoder.list_map(
            fields.get("deliveries")?,
            &fields.path("deliveries"),
            delivery,
        )?,
    })
}

fn profile_id(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProfileId, ProjectDecodeError> {
    decoder.identifier(value, path).map(ProfileId::new)
}

fn axis(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<MatrixAxis, ProjectDecodeError> {
    let fields = decoder.structure(value, "MatrixAxis", path)?;
    Ok(MatrixAxis {
        id: AxisId::new(decoder.identifier(fields.get("id")?, &fields.path("id"))?),
        values: decoder.list_map(fields.get("values")?, &fields.path("values"), axis_value)?,
    })
}

fn axis_value(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<AxisValue, ProjectDecodeError> {
    decoder.identifier(value, path).map(AxisValue::new)
}
