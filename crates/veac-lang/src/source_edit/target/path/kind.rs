use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SourceNodeKind {
    Input,
    Constant,
    Function,
    Method,
    Implementation,
    Struct,
    StructField,
    Enum,
    EnumVariant,
    EnumVariantField,
    Temporal,
    Item,
}
