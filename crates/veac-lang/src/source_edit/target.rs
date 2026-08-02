mod constructors;
mod definition;
mod path;
mod schema;
mod site;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use definition::*;
pub use path::{SourceNodeKind, SourceNodePath};

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

    pub fn constant(module: impl Into<String>, constant: impl Into<String>) -> Self {
        Self::new(
            module,
            SourceNodePath::Constant {
                constant: constant.into(),
            },
        )
    }

    pub fn component(module: impl Into<String>, component: impl Into<String>) -> Self {
        Self::new(
            module,
            SourceNodePath::Component {
                component: component.into(),
            },
        )
    }

    pub fn component_instance(module: impl Into<String>, instance: impl Into<String>) -> Self {
        Self::new(
            module,
            SourceNodePath::ComponentInstance {
                instance: instance.into(),
            },
        )
    }

    pub fn resource(module: impl Into<String>, resource: impl Into<String>) -> Self {
        Self::new(
            module,
            SourceNodePath::Resource {
                resource: resource.into(),
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
    ComponentParameterDefault {
        #[schemars(with = "schema::CanonicalNameSchema")]
        parameter: String,
    },
    ComponentInstanceArgument {
        #[schemars(with = "schema::CanonicalNameSchema")]
        parameter: String,
    },
    ComponentLocalInstanceArgument {
        #[schemars(with = "schema::CanonicalNameSchema")]
        parameter: String,
    },
    ItemRecordStart,
    ItemRecordDuration,
    ItemEnabled,
    TextContent,
    ResourceLocator,
    ModifierParameter {
        #[schemars(with = "schema::CanonicalNameSchema")]
        parameter: String,
    },
    PresetTextStyleField {
        field: SourceTextStyleField,
    },
    PresetTextLayoutField {
        field: SourceTextLayoutField,
    },
    PresetColorField {
        field: SourceColorField,
    },
    PresetAudioProcessorField {
        processor_kind: SourceAudioProcessorKind,
        field: SourceAudioProcessorField,
    },
    PresetAudioEqBandField {
        field: SourceAudioEqBandField,
    },
    PresetDeliveryField {
        field: SourceDeliveryField,
    },
}
