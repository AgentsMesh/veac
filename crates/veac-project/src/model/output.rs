use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{DeliveryId, DeliveryPathTemplate, OutputId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectOutput {
    Media {
        id: OutputId,
        media_type: MediaType,
    },
    Data {
        id: OutputId,
        schema: Option<String>,
    },
    Directory {
        id: OutputId,
    },
}

impl ProjectOutput {
    pub fn id(&self) -> &OutputId {
        match self {
            Self::Media { id, .. } | Self::Data { id, .. } | Self::Directory { id } => id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Video,
    Audio,
    Image,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectDelivery {
    File {
        id: DeliveryId,
        output: OutputId,
        destination: DeliveryPathTemplate,
    },
    Directory {
        id: DeliveryId,
        output: OutputId,
        destination: DeliveryPathTemplate,
    },
}

impl ProjectDelivery {
    pub fn id(&self) -> &DeliveryId {
        match self {
            Self::File { id, .. } | Self::Directory { id, .. } => id,
        }
    }

    pub fn output(&self) -> &OutputId {
        match self {
            Self::File { output, .. } | Self::Directory { output, .. } => output,
        }
    }

    pub fn destination(&self) -> &DeliveryPathTemplate {
        match self {
            Self::File { destination, .. } | Self::Directory { destination, .. } => destination,
        }
    }
}
