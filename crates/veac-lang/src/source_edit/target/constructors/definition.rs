use super::super::{SourceDeliveryArtifactKind, SourceNodePath, SourceNodeRef, SourcePresetKind};

impl SourceNodeRef {
    pub fn component_local_instance(
        module: impl Into<String>,
        component: impl Into<String>,
        instance: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::ComponentLocalInstance {
                component: component.into(),
                instance: instance.into(),
            },
        )
    }

    pub fn component_layer(
        module: impl Into<String>,
        component: impl Into<String>,
        layer: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::ComponentLayer {
                component: component.into(),
                layer: layer.into(),
            },
        )
    }

    pub fn component_item(
        module: impl Into<String>,
        component: impl Into<String>,
        layer: impl Into<String>,
        item: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::ComponentItem {
                component: component.into(),
                layer: layer.into(),
                item: item.into(),
            },
        )
    }

    pub fn component_modifier(
        module: impl Into<String>,
        component: impl Into<String>,
        layer: impl Into<String>,
        item: impl Into<String>,
        modifier: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::ComponentModifier {
                component: component.into(),
                layer: layer.into(),
                item: item.into(),
                modifier: modifier.into(),
            },
        )
    }

    pub fn component_apply(
        module: impl Into<String>,
        component: impl Into<String>,
        apply: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::ComponentApply {
                component: component.into(),
                apply: apply.into(),
            },
        )
    }

    pub fn component_stage(
        module: impl Into<String>,
        component: impl Into<String>,
        apply: impl Into<String>,
        stage: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::ComponentStage {
                component: component.into(),
                apply: apply.into(),
                stage: stage.into(),
            },
        )
    }

    pub fn preset(
        module: impl Into<String>,
        preset_kind: SourcePresetKind,
        preset: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::Preset {
                preset_kind,
                preset: preset.into(),
            },
        )
    }

    pub fn preset_modifier(
        module: impl Into<String>,
        preset: impl Into<String>,
        modifier: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::PresetModifier {
                preset: preset.into(),
                modifier: modifier.into(),
            },
        )
    }

    pub fn preset_stage(
        module: impl Into<String>,
        preset: impl Into<String>,
        stage: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::PresetStage {
                preset: preset.into(),
                stage: stage.into(),
            },
        )
    }

    pub fn preset_audio_processor(
        module: impl Into<String>,
        preset: impl Into<String>,
        processor: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::PresetAudioProcessor {
                preset: preset.into(),
                processor: processor.into(),
            },
        )
    }

    pub fn preset_audio_eq_band(
        module: impl Into<String>,
        preset: impl Into<String>,
        processor: impl Into<String>,
        band: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::PresetAudioEqBand {
                preset: preset.into(),
                processor: processor.into(),
                band: band.into(),
            },
        )
    }

    pub fn preset_delivery_artifact(
        module: impl Into<String>,
        preset: impl Into<String>,
        artifact_kind: SourceDeliveryArtifactKind,
        artifact: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::PresetDeliveryArtifact {
                preset: preset.into(),
                artifact_kind,
                artifact: artifact.into(),
            },
        )
    }
}
