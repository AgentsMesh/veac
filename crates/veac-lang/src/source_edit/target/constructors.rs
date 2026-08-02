use super::{SourceNodePath, SourceNodeRef};

mod definition;

impl SourceNodeRef {
    pub fn project(module: impl Into<String>, project: impl Into<String>) -> Self {
        Self::new(
            module,
            SourceNodePath::Project {
                project: project.into(),
            },
        )
    }

    pub fn sequence(
        module: impl Into<String>,
        project: impl Into<String>,
        sequence: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::Sequence {
                project: project.into(),
                sequence: sequence.into(),
            },
        )
    }

    pub fn layer(
        module: impl Into<String>,
        project: impl Into<String>,
        sequence: impl Into<String>,
        layer: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::Layer {
                project: project.into(),
                sequence: sequence.into(),
                layer: layer.into(),
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

    pub fn modifier(
        module: impl Into<String>,
        project: impl Into<String>,
        sequence: impl Into<String>,
        layer: impl Into<String>,
        item: impl Into<String>,
        modifier: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::Modifier {
                project: project.into(),
                sequence: sequence.into(),
                layer: layer.into(),
                item: item.into(),
                modifier: modifier.into(),
            },
        )
    }

    pub fn apply(
        module: impl Into<String>,
        project: impl Into<String>,
        sequence: impl Into<String>,
        apply: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::Apply {
                project: project.into(),
                sequence: sequence.into(),
                apply: apply.into(),
            },
        )
    }

    pub fn stage(
        module: impl Into<String>,
        project: impl Into<String>,
        sequence: impl Into<String>,
        apply: impl Into<String>,
        stage: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::Stage {
                project: project.into(),
                sequence: sequence.into(),
                apply: apply.into(),
                stage: stage.into(),
            },
        )
    }
}
