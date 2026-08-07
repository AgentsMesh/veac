use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ReservedLiteral {
    False,
    True,
}

impl ReservedLiteral {
    pub const ALL: [Self; 2] = [Self::False, Self::True];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::False => "false",
            Self::True => "true",
        }
    }

    pub const fn value(self) -> bool {
        match self {
            Self::False => false,
            Self::True => true,
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|literal| literal.as_str() == value)
    }
}

pub fn is_reserved_literal(value: &str) -> bool {
    ReservedLiteral::parse(value).is_some()
}
