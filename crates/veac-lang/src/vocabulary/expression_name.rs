use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::LanguageLayer;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ExpressionNameKind {
    Function,
    Parameter,
    Local,
}

impl ExpressionNameKind {
    pub const ALL: [Self; 3] = [Self::Function, Self::Parameter, Self::Local];
}

pub fn accepts_expression_name(spelling: &str, kind: ExpressionNameKind) -> bool {
    if spelling == "self" || !crate::name::is_name(spelling) {
        return false;
    }
    let is_control = super::control::all_uses().any(|usage| {
        usage.layer() == LanguageLayer::ExecutableExpression && usage.matches(spelling)
    });
    match kind {
        ExpressionNameKind::Function
        | ExpressionNameKind::Parameter
        | ExpressionNameKind::Local => !is_control,
    }
}
