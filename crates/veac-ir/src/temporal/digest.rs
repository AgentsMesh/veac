use std::fmt;

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{
    TemporalInputDeclaration, TemporalNode, TemporalNodeId, TemporalProgram, TemporalType,
};

const DIGEST_DOMAIN: &str = "veac.temporal-program.v1";

#[derive(Debug)]
pub enum TemporalDigestError {
    Json(serde_json::Error),
    NonCanonicalNumber,
}

impl fmt::Display for TemporalDigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(
                formatter,
                "temporal program canonicalization failed: {error}"
            ),
            Self::NonCanonicalNumber => {
                formatter.write_str("temporal program contains a non-canonical number")
            }
        }
    }
}

impl std::error::Error for TemporalDigestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::NonCanonicalNumber => None,
        }
    }
}

#[derive(Serialize)]
struct DigestContent<'a> {
    domain: &'static str,
    opset_version: u16,
    inputs: &'a [TemporalInputDeclaration],
    result_type: TemporalType,
    nodes: &'a [TemporalNode],
    result: TemporalNodeId,
}

pub fn temporal_program_digest(program: &TemporalProgram) -> Result<String, TemporalDigestError> {
    if !numbers_valid(program) {
        return Err(TemporalDigestError::NonCanonicalNumber);
    }
    let mut nodes = program.nodes.clone();
    for node in &mut nodes {
        node.provenance_id = None;
    }
    let content = DigestContent {
        domain: DIGEST_DOMAIN,
        opset_version: program.opset_version,
        inputs: &program.inputs,
        result_type: program.result_type,
        nodes: &nodes,
        result: program.result,
    };
    let bytes = serde_json_canonicalizer::to_vec(&content).map_err(TemporalDigestError::Json)?;
    Ok(hex(Sha256::digest(bytes)))
}

fn numbers_valid(program: &TemporalProgram) -> bool {
    program.nodes.iter().all(|node| match &node.kind {
        super::TemporalNodeKind::Literal { value } => value_numbers_valid(value),
        super::TemporalNodeKind::CurveSample { keys, .. } => keys.iter().all(|key| {
            position_valid(key.position)
                && value_numbers_valid(&key.value)
                && interpolation_valid(&key.interpolation)
        }),
        _ => true,
    })
}

fn position_valid(value: super::TemporalCurvePosition) -> bool {
    match value {
        super::TemporalCurvePosition::Scalar { value } => number_valid(value),
        super::TemporalCurvePosition::Time { .. } => true,
    }
}

fn value_numbers_valid(value: &super::TemporalValue) -> bool {
    match value {
        super::TemporalValue::Scalar { value } | super::TemporalValue::Angle { degrees: value } => {
            number_valid(*value)
        }
        super::TemporalValue::Length { value } => number_valid(value.value),
        super::TemporalValue::Vec2 { value } => number_valid(value.x) && number_valid(value.y),
        super::TemporalValue::Point { value } => {
            number_valid(value.x.value) && number_valid(value.y.value)
        }
        super::TemporalValue::Rect { value } => [value.x, value.y, value.width, value.height]
            .into_iter()
            .all(number_valid),
        _ => true,
    }
}

fn interpolation_valid(value: &crate::Interpolation) -> bool {
    match value {
        crate::Interpolation::Spring {
            frequency,
            decay,
            initial_velocity,
        } => [*frequency, *decay, *initial_velocity]
            .into_iter()
            .all(number_valid),
        crate::Interpolation::CubicBezier { x1, y1, x2, y2 } => {
            [*x1, *y1, *x2, *y2].into_iter().all(number_valid)
        }
        _ => true,
    }
}

fn number_valid(value: f64) -> bool {
    value.is_finite() && !(value == 0.0 && value.is_sign_negative())
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.as_ref().len() * 2);
    for byte in bytes.as_ref() {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
