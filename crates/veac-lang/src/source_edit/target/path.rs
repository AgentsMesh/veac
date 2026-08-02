use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::definition::{SourceDeliveryArtifactKind, SourcePresetKind};

mod identity;
mod kind;

pub use kind::SourceNodeKind;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceNodePath {
    Project {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
    },
    Constant {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        constant: String,
    },
    Component {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        component: String,
    },
    ComponentInstance {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        instance: String,
    },
    ComponentLocalInstance {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        component: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        instance: String,
    },
    ComponentLayer {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        component: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        layer: String,
    },
    ComponentItem {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        component: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        layer: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        item: String,
    },
    ComponentModifier {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        component: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        layer: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        item: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        modifier: String,
    },
    ComponentApply {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        component: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        apply: String,
    },
    ComponentStage {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        component: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        apply: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        stage: String,
    },
    Preset {
        preset_kind: SourcePresetKind,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        preset: String,
    },
    PresetModifier {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        preset: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        modifier: String,
    },
    PresetStage {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        preset: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        stage: String,
    },
    PresetAudioProcessor {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        preset: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        processor: String,
    },
    PresetAudioEqBand {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        preset: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        processor: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        band: String,
    },
    PresetDeliveryArtifact {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        preset: String,
        artifact_kind: SourceDeliveryArtifactKind,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        artifact: String,
    },
    Resource {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        resource: String,
    },
    Sequence {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        sequence: String,
    },
    Layer {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        sequence: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        layer: String,
    },
    Item {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        sequence: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        layer: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        item: String,
    },
    Modifier {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        sequence: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        layer: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        item: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        modifier: String,
    },
    Apply {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        sequence: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        apply: String,
    },
    Stage {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        sequence: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        apply: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        stage: String,
    },
}
