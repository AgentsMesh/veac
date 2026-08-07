mod constructors;
mod declaration;
mod expression_path;
mod path;
mod schema;
mod site;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use declaration::*;
pub use expression_path::*;
pub use path::kind::SourceNodeKind;
pub use path::SourceNodePath;
pub use site::StatementSite;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SourceNodeRef {
    pub module: String,
    pub path: SourceNodePath,
}

impl SourceNodeRef {
    pub fn new(module: impl Into<String>, path: SourceNodePath) -> Self {
        Self {
            module: module.into(),
            path,
        }
    }

    pub fn input(module: impl Into<String>, input: impl Into<String>) -> Self {
        Self::new(
            module,
            SourceNodePath::Input {
                input: input.into(),
            },
        )
    }

    pub fn constant(module: impl Into<String>, constant: impl Into<String>) -> Self {
        Self::new(
            module,
            SourceNodePath::Constant {
                constant: constant.into(),
            },
        )
    }

    pub fn function(module: impl Into<String>, function: impl Into<String>) -> Self {
        Self::new(
            module,
            SourceNodePath::Function {
                function: function.into(),
            },
        )
    }

    pub fn method(
        module: impl Into<String>,
        receiver: impl Into<String>,
        method: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::Method {
                receiver: receiver.into(),
                method: method.into(),
            },
        )
    }

    pub fn implementation(
        module: impl Into<String>,
        receiver: impl Into<String>,
        implementation: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::Implementation {
                receiver: receiver.into(),
                implementation: implementation.into(),
            },
        )
    }

    pub fn kind(&self) -> SourceNodeKind {
        self.path.kind()
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpressionSite {
    ConstantValue,
    BodyExpression { path: SourceExpressionPath },
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum BodySite {
    FunctionBody,
    MethodBody,
    TemporalAnimation {
        property: SourceTemporalProperty,
    },
    ComponentAnimation {
        ordinal: u32,
        property: SourceTemporalProperty,
    },
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SourceTemporalProperty {
    VisualPosition,
    VisualScale,
    VisualRotation,
    VisualCrop,
    VisualOpacity,
    AudioGain,
    AudioPan,
    MaskPosition,
    MaskScale,
    MaskRotation,
    MaskFeather,
    MaskExpansion,
    TextPosition,
    TextScale,
    TextRotation,
    TextReveal,
    TextHighlightProgress,
    TextOpacity,
    EffectParameter,
    ApplyOpacity,
}
