use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{exact, BuildInputsError};
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
            Self::Enum { .. } => return None,
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
            Self::Time { value } => unit_value(value, PrimitiveType::Time),
            Self::Length { value } => unit_value(value, PrimitiveType::Length),
            Self::Angle { value } => unit_value(value, PrimitiveType::Angle),
            Self::Color { value } if valid_color(value) => {
                Some(Value::Color(value.to_ascii_lowercase().into()))
            }
            Self::Color { .. } => None,
            Self::Enum { .. } => None,
        }
        .ok_or_else(|| {
            BuildInputsError::new(
                "PROGRAM_INPUT_VALUE",
                format!("invalid {} Build input literal", self.type_label()),
            )
        })
    }

    pub(crate) fn matches_type(&self, expected: &ValueType) -> bool {
        match self {
            Self::Enum { .. } => matches!(expected.kind(), ValueTypeKind::Nominal(_)),
            _ => self
                .primitive_type()
                .is_some_and(|actual| expected.as_primitive() == Some(actual)),
        }
    }

    pub(crate) fn bound_value(
        &self,
        expected: &ValueType,
        registry: &TypeRegistry,
    ) -> Result<Value, BuildInputsError> {
        self.validate_shape()?;
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
        self.primitive_type().map_or("enum", PrimitiveType::as_str)
    }
}

pub(crate) fn type_mismatch(expected: &ValueType, actual: &str) -> BuildInputsError {
    BuildInputsError::new(
        "PROGRAM_INPUT_TYPE_MISMATCH",
        format!("Build input expects {expected}, got {actual}"),
    )
}

fn unit_value(raw: &str, kind: PrimitiveType) -> Option<Value> {
    let exact = match kind {
        PrimitiveType::Time => exact::unit(raw, "s", (1, 1))
            .or_else(|| exact::unit(raw, "ms", (1, 1_000)))
            .or_else(|| exact::unit(raw, "us", (1, 1_000_000))),
        PrimitiveType::Length => exact::unit(raw, "px", (1, 1)),
        PrimitiveType::Angle => exact::unit(raw, "deg", (1, 1)),
        _ => None,
    }?;
    match kind {
        PrimitiveType::Time => Some(Value::Time(exact)),
        PrimitiveType::Length => Some(Value::Length(exact)),
        PrimitiveType::Angle => Some(Value::Angle(exact)),
        _ => None,
    }
}

fn valid_color(value: &str) -> bool {
    matches!(value.len(), 7 | 9)
        && value.starts_with('#')
        && value[1..].bytes().all(|value| value.is_ascii_hexdigit())
}
