use std::collections::BTreeMap;

use veac_lang::program::expression::Value;

use crate::{AxisId, AxisValue, InstanceSelector, LocaleId, ProfileId, TargetId, TargetRef};

use super::super::ProjectDecodeError;
use super::value::{unknown_variant, Decoder};

pub(super) fn target_ref(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<TargetRef, ProjectDecodeError> {
    let fields = decoder.structure(value, "TargetRef", path)?;
    Ok(TargetRef {
        target: TargetId::new(decoder.identifier(fields.get("target")?, &fields.path("target"))?),
        profile: decoder
            .optional_identifier(fields.get("profile")?, &fields.path("profile"))?
            .map(ProfileId::new),
        selector: selector(decoder, fields.get("selector")?, &fields.path("selector"))?,
    })
}

fn selector(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<InstanceSelector, ProjectDecodeError> {
    let variant = decoder.variant(value, "InstanceSelector", path)?;
    match variant.name {
        "Same" => Ok(InstanceSelector::Same {}),
        "AllMatching" => Ok(InstanceSelector::AllMatching {}),
        "Exact" => {
            let values = decoder.list_map(
                variant.fields.get("axes")?,
                &variant.fields.path("axes"),
                matrix_value,
            )?;
            let axes = collect_axes(values, &variant.fields.path("axes"))?;
            Ok(InstanceSelector::Exact {
                locale: decoder
                    .optional_identifier(
                        variant.fields.get("locale")?,
                        &variant.fields.path("locale"),
                    )?
                    .map(LocaleId::new),
                axes,
            })
        }
        name => Err(unknown_variant(path, "InstanceSelector", name)),
    }
}

fn matrix_value(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<(AxisId, AxisValue), ProjectDecodeError> {
    let fields = decoder.structure(value, "MatrixValue", path)?;
    Ok((
        AxisId::new(decoder.identifier(fields.get("axis")?, &fields.path("axis"))?),
        AxisValue::new(decoder.identifier(fields.get("value")?, &fields.path("value"))?),
    ))
}

fn collect_axes(
    values: Vec<(AxisId, AxisValue)>,
    path: &str,
) -> Result<BTreeMap<AxisId, AxisValue>, ProjectDecodeError> {
    let mut result = BTreeMap::new();
    for (axis, value) in values {
        if result.insert(axis.clone(), value).is_some() {
            return Err(ProjectDecodeError::new(
                path,
                format!("matrix axis `{axis}` is assigned more than once"),
            ));
        }
    }
    Ok(result)
}
