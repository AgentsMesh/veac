use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum LanguageLayer {
    StaticProgram,
    #[schemars(skip)]
    CoreAuthoring,
    ExecutableExpression,
}

impl LanguageLayer {
    pub const ALL: [Self; 2] = [Self::StaticProgram, Self::ExecutableExpression];

    pub const fn is_public(self) -> bool {
        !matches!(self, Self::CoreAuthoring)
    }
}
