use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum VocabularyCategory {
    ReservedLiteral,
    ContextualControl,
    EnumValue,
    BuiltinFunction,
    UnitSuffix,
    ValueType,
}

impl VocabularyCategory {
    pub const ALL: [Self; 6] = [
        Self::ReservedLiteral,
        Self::ContextualControl,
        Self::EnumValue,
        Self::BuiltinFunction,
        Self::UnitSuffix,
        Self::ValueType,
    ];
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum IdentifierPolicy {
    Allowed,
    Reserved,
    NotApplicable,
}

impl IdentifierPolicy {
    pub const ALL: [Self; 3] = [Self::Allowed, Self::Reserved, Self::NotApplicable];
}
