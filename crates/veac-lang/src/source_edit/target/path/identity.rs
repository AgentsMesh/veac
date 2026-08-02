use super::{SourceNodeKind, SourceNodePath};

impl SourceNodePath {
    pub fn kind(&self) -> SourceNodeKind {
        match self {
            Self::Project { .. } => SourceNodeKind::Project,
            Self::Constant { .. } => SourceNodeKind::Constant,
            Self::Component { .. } => SourceNodeKind::Component,
            Self::ComponentInstance { .. } => SourceNodeKind::ComponentInstance,
            Self::ComponentLocalInstance { .. } => SourceNodeKind::ComponentLocalInstance,
            Self::ComponentLayer { .. } => SourceNodeKind::ComponentLayer,
            Self::ComponentItem { .. } => SourceNodeKind::ComponentItem,
            Self::ComponentModifier { .. } => SourceNodeKind::ComponentModifier,
            Self::ComponentApply { .. } => SourceNodeKind::ComponentApply,
            Self::ComponentStage { .. } => SourceNodeKind::ComponentStage,
            Self::Preset { .. } => SourceNodeKind::Preset,
            Self::PresetModifier { .. } => SourceNodeKind::PresetModifier,
            Self::PresetStage { .. } => SourceNodeKind::PresetStage,
            Self::PresetAudioProcessor { .. } => SourceNodeKind::PresetAudioProcessor,
            Self::PresetAudioEqBand { .. } => SourceNodeKind::PresetAudioEqBand,
            Self::PresetDeliveryArtifact { .. } => SourceNodeKind::PresetDeliveryArtifact,
            Self::Resource { .. } => SourceNodeKind::Resource,
            Self::Sequence { .. } => SourceNodeKind::Sequence,
            Self::Layer { .. } => SourceNodeKind::Layer,
            Self::Item { .. } => SourceNodeKind::Item,
            Self::Modifier { .. } => SourceNodeKind::Modifier,
            Self::Apply { .. } => SourceNodeKind::Apply,
            Self::Stage { .. } => SourceNodeKind::Stage,
        }
    }

    pub(crate) fn identifiers(&self) -> Vec<&str> {
        match self {
            Self::Project { project } => vec![project],
            Self::Constant { constant } => vec![constant],
            Self::Component { component } => vec![component],
            Self::ComponentInstance { instance } => vec![instance],
            Self::ComponentLocalInstance {
                component,
                instance,
            } => vec![component, instance],
            Self::ComponentLayer { component, layer } => vec![component, layer],
            Self::ComponentItem {
                component,
                layer,
                item,
            } => vec![component, layer, item],
            Self::ComponentModifier {
                component,
                layer,
                item,
                modifier,
            } => vec![component, layer, item, modifier],
            Self::ComponentApply { component, apply } => vec![component, apply],
            Self::ComponentStage {
                component,
                apply,
                stage,
            } => vec![component, apply, stage],
            Self::Preset { preset, .. } => vec![preset],
            Self::PresetAudioProcessor { preset, processor } => vec![preset, processor],
            Self::PresetAudioEqBand {
                preset,
                processor,
                band,
            } => vec![preset, processor, band],
            Self::PresetModifier { preset, modifier } => vec![preset, modifier],
            Self::PresetStage { preset, stage } => vec![preset, stage],
            Self::PresetDeliveryArtifact {
                preset, artifact, ..
            } => vec![preset, artifact],
            Self::Resource { resource } => vec![resource],
            Self::Sequence { project, sequence } => vec![project, sequence],
            Self::Layer {
                project,
                sequence,
                layer,
            } => vec![project, sequence, layer],
            Self::Item {
                project,
                sequence,
                layer,
                item,
            } => vec![project, sequence, layer, item],
            Self::Modifier {
                project,
                sequence,
                layer,
                item,
                modifier,
            } => vec![project, sequence, layer, item, modifier],
            Self::Apply {
                project,
                sequence,
                apply,
            } => vec![project, sequence, apply],
            Self::Stage {
                project,
                sequence,
                apply,
                stage,
            } => vec![project, sequence, apply, stage],
        }
    }
}
