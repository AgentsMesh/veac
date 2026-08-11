use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{BuildInputManifestValue, BuildInputsError};
use crate::program::expression::{PrimitiveType, Value, ValueType, ValueTypeKind};
use crate::program::{TypeDefinitionKind, TypeRegistry};

const MATERIAL_BINDING: &str = "MaterialBinding";
const FIELDS: [(&str, PrimitiveType); 7] = [
    ("kind", PrimitiveType::Text),
    ("path", PrimitiveType::Text),
    ("sha256", PrimitiveType::Text),
    ("authority", PrimitiveType::Text),
    ("artifact_key", PrimitiveType::Text),
    ("video_stream", PrimitiveType::Integer),
    ("audio_stream", PrimitiveType::Integer),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MaterialInputKind {
    Video,
    Audio,
    Image,
    Font,
    #[serde(rename = "lut_1d")]
    Lut1d,
    #[serde(rename = "lut_3d")]
    Lut3d,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum MaterialInputAuthority {
    ProjectMaterial,
    Artifact { artifact_key: String },
}

impl MaterialInputKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Image => "image",
            Self::Font => "font",
            Self::Lut1d => "lut_1d",
            Self::Lut3d => "lut_3d",
        }
    }
}

pub(crate) fn matches(expected: &ValueType, registry: &TypeRegistry) -> bool {
    let ValueTypeKind::Nominal(reference) = expected.kind() else {
        return false;
    };
    let Some(definition) = registry.definition(reference.id()) else {
        return false;
    };
    let TypeDefinitionKind::Struct(layout) = definition.kind() else {
        return false;
    };
    definition.declared_name() == MATERIAL_BINDING
        && layout.fields().len() == FIELDS.len()
        && layout
            .fields()
            .iter()
            .zip(FIELDS)
            .all(|(actual, expected)| {
                actual.name() == expected.0
                    && actual.value_type().as_primitive() == Some(expected.1)
            })
}

pub(crate) fn validate(value: &BuildInputManifestValue) -> Result<(), BuildInputsError> {
    let BuildInputManifestValue::Material {
        path,
        sha256,
        authority,
        ..
    } = value
    else {
        return Err(error("material validator received a non-material value"));
    };
    if !valid_path(path) {
        return Err(error(
            "material path must be canonical, relative, slash-separated, and traversal-free",
        ));
    }
    if !valid_digest(sha256) {
        return Err(error(
            "material sha256 must be 64 lowercase hexadecimal bytes",
        ));
    }
    if let MaterialInputAuthority::Artifact { artifact_key } = authority {
        if !valid_digest(artifact_key) {
            return Err(error(
                "material artifact_key must be 64 lowercase hexadecimal bytes",
            ));
        }
    }
    Ok(())
}

pub(crate) fn value(
    input: &BuildInputManifestValue,
    expected: &ValueType,
    registry: &TypeRegistry,
) -> Result<Value, BuildInputsError> {
    validate(input)?;
    if !matches(expected, registry) {
        return Err(super::value::type_mismatch(expected, "material"));
    }
    let BuildInputManifestValue::Material {
        kind,
        path,
        sha256,
        authority,
        video_stream,
        audio_stream,
    } = input
    else {
        unreachable!("validated material input has the material variant")
    };
    let (authority, artifact_key) = match authority {
        MaterialInputAuthority::ProjectMaterial => ("project_material", ""),
        MaterialInputAuthority::Artifact { artifact_key } => ("artifact", artifact_key.as_str()),
    };
    let ValueTypeKind::Nominal(reference) = expected.kind() else {
        unreachable!("material shape verification requires a nominal type")
    };
    Value::structure(
        registry,
        reference.id(),
        vec![
            text(kind.as_str()),
            text(path),
            text(sha256),
            text(authority),
            text(artifact_key),
            Value::Integer(stream(*video_stream)),
            Value::Integer(stream(*audio_stream)),
        ],
    )
    .map_err(|cause| BuildInputsError::new("PROGRAM_INPUT_VALUE", cause.message()))
}

fn text(value: &str) -> Value {
    Value::Text(Arc::from(value))
}

fn stream(value: Option<u32>) -> i64 {
    value.map_or(-1, i64::from)
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_path(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.contains('\\')
        && !value.chars().any(char::is_control)
        && value.as_bytes().get(1).copied() != Some(b':')
        && value
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn error(message: impl Into<String>) -> BuildInputsError {
    BuildInputsError::new("PROGRAM_INPUT_MATERIAL", message)
}
