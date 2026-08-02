use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SourceNodeKind {
    Project,
    Constant,
    Component,
    ComponentInstance,
    ComponentLocalInstance,
    ComponentLayer,
    ComponentItem,
    ComponentModifier,
    ComponentApply,
    ComponentStage,
    Preset,
    PresetModifier,
    PresetStage,
    PresetAudioProcessor,
    PresetAudioEqBand,
    PresetDeliveryArtifact,
    Resource,
    Sequence,
    Layer,
    Item,
    Modifier,
    Apply,
    Stage,
}
