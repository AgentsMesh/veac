use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    exact, material, primitive, BuildInputRole, BuildInputsError, MaterialInputAuthority,
    MaterialInputKind,
};
use crate::program::expression::{
    PrimitiveType, Value, ValueType, ValueTypeKind, MAX_TEXT_VALUE_BYTES,
};
use crate::program::{TypeDefinitionKind, TypeRegistry};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum BuildInputManifestValue {
    Bool {
        value: bool,
    },
    #[serde(rename = "int")]
    Integer {
        value: i64,
    },
    Scalar {
        value: String,
    },
    Text {
        #[schemars(extend("x-veac-max-utf8-bytes" = MAX_TEXT_VALUE_BYTES))]
        value: String,
    },
    Time {
        value: String,
    },
    Length {
        value: String,
    },
    Angle {
        value: String,
    },
    Color {
        value: String,
    },
    Enum {
        #[schemars(
            length(min = 1, max = 128),
            regex(pattern = r"^[A-Za-z_][A-Za-z0-9_]*(-[A-Za-z0-9_]+)*$")
        )]
        value: String,
    },
    Material {
        kind: MaterialInputKind,
        path: String,
        sha256: String,
        authority: MaterialInputAuthority,
        video_stream: Option<u32>,
        audio_stream: Option<u32>,
    },
}

impl BuildInputManifestValue {
    pub fn value_type(&self) -> Option<ValueType> {
        self.primitive_type().map(ValueType::from)
    }

    pub fn primitive_type(&self) -> Option<PrimitiveType> {
        Some(match self {
            Self::Bool { .. } => PrimitiveType::Boolean,
            Self::Integer { .. } => PrimitiveType::Integer,
            Self::Scalar { .. } => PrimitiveType::Scalar,
            Self::Text { .. } => PrimitiveType::Text,
            Self::Time { .. } => PrimitiveType::Time,
            Self::Length { .. } => PrimitiveType::Length,
            Self::Angle { .. } => PrimitiveType::Angle,
            Self::Color { .. } => PrimitiveType::Color,
            Self::Enum { .. } | Self::Material { .. } => return None,
        })
    }

    pub(crate) fn runtime_value(&self) -> Result<Value, BuildInputsError> {
        match self {
            Self::Bool { value } => Some(Value::Bool(*value)),
            Self::Integer { value } => Some(Value::Integer(*value)),
            Self::Scalar { value } => exact::decimal(value).map(Value::Scalar),
            Self::Text { value } if value.len() <= MAX_TEXT_VALUE_BYTES => {
                Some(Value::Text(Arc::from(value.as_str())))
            }
            Self::Text { .. } => None,
            Self::Time { value } => primitive::unit_value(value, PrimitiveType::Time),
            Self::Length { value } => primitive::unit_value(value, PrimitiveType::Length),
            Self::Angle { value } => primitive::unit_value(value, PrimitiveType::Angle),
            Self::Color { value } if primitive::valid_color(value) => {
                Some(Value::Color(value.to_ascii_lowercase().into()))
            }
            Self::Color { .. } => None,
            Self::Enum { .. } | Self::Material { .. } => None,
        }
        .ok_or_else(|| {
            BuildInputsError::new(
                "PROGRAM_INPUT_VALUE",
                format!("invalid {} Build input literal", self.type_label()),
            )
        })
    }

    pub(crate) fn matches_declaration(
        &self,
        role: BuildInputRole,
        expected: &ValueType,
        registry: &TypeRegistry,
    ) -> bool {
        match self {
            Self::Material { .. } => {
                role == BuildInputRole::Material && material::matches(expected, registry)
            }
            Self::Enum { .. } => {
                role != BuildInputRole::Material
                    && matches!(expected.kind(), ValueTypeKind::Nominal(_))
            }
            _ => self.primitive_type().is_some_and(|actual| {
                role != BuildInputRole::Material && expected.as_primitive() == Some(actual)
            }),
        }
    }

    pub(crate) fn bound_value(
        &self,
        expected: &ValueType,
        registry: &TypeRegistry,
    ) -> Result<Value, BuildInputsError> {
        self.validate_shape()?;
        if matches!(self, Self::Material { .. }) {
            return material::value(self, expected, registry);
        }
        let Self::Enum { value } = self else {
            return self.runtime_value();
        };
        let ValueTypeKind::Nominal(reference) = expected.kind() else {
            return Err(type_mismatch(expected, self.type_label()));
        };
        let definition = registry.definition(reference.id()).ok_or_else(|| {
            BuildInputsError::new(
                "PROGRAM_INPUT_NOMINAL_TYPE",
                format!("Build input type `{reference}` is outside the verified registry"),
            )
        })?;
        let TypeDefinitionKind::Enum(layout) = definition.kind() else {
            return Err(BuildInputsError::new(
                "PROGRAM_INPUT_NOMINAL_KIND",
                format!("Build input type `{reference}` is not an enum"),
            ));
        };
        if layout
            .variants()
            .iter()
            .any(|variant| !variant.fields().is_empty())
        {
            return Err(BuildInputsError::new(
                "PROGRAM_INPUT_ENUM_PAYLOAD",
                format!("Build input enum `{reference}` must not contain payload fields"),
            ));
        }
        let variant = layout.variant(value).ok_or_else(|| {
            BuildInputsError::new(
                "PROGRAM_INPUT_ENUM_VARIANT",
                format!("unknown `{reference}` Build input variant `{value}`"),
            )
        })?;
        Value::variant(registry, reference.id(), variant.index(), Vec::new()).map_err(|error| {
            BuildInputsError::new("PROGRAM_INPUT_VALUE", error.message().to_owned())
        })
    }

    pub(crate) fn type_label(&self) -> &'static str {
        match self {
            Self::Material { .. } => "material",
            _ => self.primitive_type().map_or("enum", PrimitiveType::as_str),
        }
    }
}

pub(crate) fn type_mismatch(expected: &ValueType, actual: &str) -> BuildInputsError {
    BuildInputsError::new(
        "PROGRAM_INPUT_TYPE_MISMATCH",
        format!("Build input expects {expected}, got {actual}"),
    )
}
