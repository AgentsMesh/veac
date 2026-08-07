use super::{SourceNodePath, SourceNodeRef, SourceTemporalProperty};

mod nominal;

impl SourceNodeRef {
    pub fn temporal(
        module: impl Into<String>,
        project: impl Into<String>,
        sequence: impl Into<String>,
        layer: impl Into<String>,
        item: impl Into<String>,
        property: SourceTemporalProperty,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::Temporal {
                project: project.into(),
                sequence: sequence.into(),
                layer: layer.into(),
                item: item.into(),
                property,
            },
        )
    }

    pub fn item(
        module: impl Into<String>,
        project: impl Into<String>,
        sequence: impl Into<String>,
        layer: impl Into<String>,
        item: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::Item {
                project: project.into(),
                sequence: sequence.into(),
                layer: layer.into(),
                item: item.into(),
            },
        )
    }

    pub fn temporal_clip_mask(
        module: impl Into<String>,
        path: [&str; 4],
        mask_index: u32,
        property: SourceTemporalProperty,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::TemporalClipMask {
                project: path[0].to_owned(),
                sequence: path[1].to_owned(),
                layer: path[2].to_owned(),
                item: path[3].to_owned(),
                mask_index,
                property,
            },
        )
    }

    pub fn temporal_clip_effect(
        module: impl Into<String>,
        path: [&str; 4],
        effect: impl Into<String>,
        parameter: impl Into<String>,
        property: SourceTemporalProperty,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::TemporalClipEffect {
                project: path[0].to_owned(),
                sequence: path[1].to_owned(),
                layer: path[2].to_owned(),
                item: path[3].to_owned(),
                effect: effect.into(),
                parameter: parameter.into(),
                property,
            },
        )
    }

    pub fn temporal_apply(
        module: impl Into<String>,
        path: [&str; 3],
        property: SourceTemporalProperty,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::TemporalApply {
                project: path[0].to_owned(),
                sequence: path[1].to_owned(),
                apply: path[2].to_owned(),
                property,
            },
        )
    }

    pub fn temporal_apply_mask(
        module: impl Into<String>,
        path: [&str; 3],
        mask_index: u32,
        property: SourceTemporalProperty,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::TemporalApplyMask {
                project: path[0].to_owned(),
                sequence: path[1].to_owned(),
                apply: path[2].to_owned(),
                mask_index,
                property,
            },
        )
    }

    pub fn temporal_apply_effect(
        module: impl Into<String>,
        path: [&str; 3],
        tail: [&str; 3],
        property: SourceTemporalProperty,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::TemporalApplyEffect {
                project: path[0].to_owned(),
                sequence: path[1].to_owned(),
                apply: path[2].to_owned(),
                stage: tail[0].to_owned(),
                effect: tail[1].to_owned(),
                parameter: tail[2].to_owned(),
                property,
            },
        )
    }
}
