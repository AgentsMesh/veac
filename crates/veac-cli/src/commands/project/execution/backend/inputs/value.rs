use veac_lang::program::expression::{PrimitiveType, ValueTypeKind};
use veac_lang::program::{BuildInputDeclaration, BuildInputManifestValue};
use veac_project::{ProjectLiteral, ProjectRational};

use super::super::{failed, ProjectOptionExt};

pub(super) fn literal(
    literal: &ProjectLiteral,
    declaration: &BuildInputDeclaration,
) -> Result<BuildInputManifestValue, veac_build::ProjectBackendError> {
    Ok(match literal {
        ProjectLiteral::Bool { value } => BuildInputManifestValue::Bool { value: *value },
        ProjectLiteral::Integer { value } => BuildInputManifestValue::Integer { value: *value },
        ProjectLiteral::Scalar { value } => BuildInputManifestValue::Scalar {
            value: decimal(*value)?,
        },
        ProjectLiteral::Text { value } => BuildInputManifestValue::Text {
            value: value.clone(),
        },
        ProjectLiteral::Identifier { value } => identifier(value, declaration)?,
        ProjectLiteral::Duration { value } => BuildInputManifestValue::Time {
            value: format!("{}s", decimal(*value)?),
        },
    })
}

pub(super) fn identifier(
    value: &str,
    declaration: &BuildInputDeclaration,
) -> Result<BuildInputManifestValue, veac_build::ProjectBackendError> {
    match declaration.value_type().kind() {
        ValueTypeKind::Primitive(PrimitiveType::Text) => Ok(BuildInputManifestValue::Text {
            value: value.to_owned(),
        }),
        ValueTypeKind::Nominal(_) => Ok(BuildInputManifestValue::Enum {
            value: value.to_owned(),
        }),
        _ => Err(failed(format!(
            "project identifier input '{}' requires text or a payloadless enum",
            declaration.name()
        ))),
    }
}

fn decimal(value: ProjectRational) -> Result<String, veac_build::ProjectBackendError> {
    let mut denominator = value.denominator;
    if denominator == 0 {
        return Err(failed("project rational denominator must be positive"));
    }
    let mut twos = 0_u32;
    let mut fives = 0_u32;
    while denominator % 2 == 0 {
        denominator /= 2;
        twos += 1;
    }
    while denominator % 5 == 0 {
        denominator /= 5;
        fives += 1;
    }
    if denominator != 1 {
        return Err(failed(
            "project rational cannot be represented by an exact Build input decimal",
        ));
    }
    let scale = twos.max(fives);
    let factor = 2_i128
        .checked_pow(scale - twos)
        .and_then(|value| value.checked_mul(5_i128.checked_pow(scale - fives)?))
        .project_required("project rational exceeds the Build input decimal range")?;
    let scaled = i128::from(value.numerator)
        .checked_mul(factor)
        .project_required("project rational exceeds the Build input decimal range")?;
    render_decimal(scaled, scale)
}

fn render_decimal(value: i128, scale: u32) -> Result<String, veac_build::ProjectBackendError> {
    if scale == 0 {
        return Ok(value.to_string());
    }
    let denominator = 10_i128
        .checked_pow(scale)
        .project_required("project rational exceeds the Build input decimal range")?;
    let sign = if value < 0 { "-" } else { "" };
    let magnitude = value.unsigned_abs();
    let whole = magnitude / denominator as u128;
    let fraction = magnitude % denominator as u128;
    Ok(format!(
        "{sign}{whole}.{fraction:0width$}",
        width = scale as usize
    ))
}

#[cfg(test)]
#[path = "value/tests.rs"]
mod tests;
