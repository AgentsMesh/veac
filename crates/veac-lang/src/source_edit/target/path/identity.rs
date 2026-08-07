use super::{kind::SourceNodeKind, SourceNodePath};

impl SourceNodePath {
    pub fn kind(&self) -> SourceNodeKind {
        match self {
            Self::Input { .. } => SourceNodeKind::Input,
            Self::Constant { .. } => SourceNodeKind::Constant,
            Self::Function { .. } => SourceNodeKind::Function,
            Self::Method { .. } => SourceNodeKind::Method,
            Self::Implementation { .. } => SourceNodeKind::Implementation,
            Self::Struct { .. } => SourceNodeKind::Struct,
            Self::StructField { .. } => SourceNodeKind::StructField,
            Self::Enum { .. } => SourceNodeKind::Enum,
            Self::EnumVariant { .. } => SourceNodeKind::EnumVariant,
            Self::EnumVariantField { .. } => SourceNodeKind::EnumVariantField,
            Self::Temporal { .. }
            | Self::TemporalClipMask { .. }
            | Self::TemporalClipEffect { .. }
            | Self::TemporalApply { .. }
            | Self::TemporalApplyMask { .. }
            | Self::TemporalApplyEffect { .. } => SourceNodeKind::Temporal,
            Self::Item { .. } => SourceNodeKind::Item,
        }
    }

    pub(crate) fn identifiers(&self) -> Vec<&str> {
        match self {
            Self::Input { input } => vec![input],
            Self::Constant { constant } => vec![constant],
            Self::Function { function } => vec![function],
            Self::Method { receiver, method } => vec![receiver, method],
            Self::Implementation {
                receiver,
                implementation,
            } => vec![receiver, implementation],
            Self::Struct { structure } => vec![structure],
            Self::StructField { structure, field } => vec![structure, field],
            Self::Enum { enumeration } => vec![enumeration],
            Self::EnumVariant {
                enumeration,
                variant,
            } => vec![enumeration, variant],
            Self::EnumVariantField {
                enumeration,
                variant,
                field,
            } => vec![enumeration, variant, field],
            Self::Temporal {
                project,
                sequence,
                layer,
                item,
                ..
            } => vec![project, sequence, layer, item],
            Self::TemporalClipMask {
                project,
                sequence,
                layer,
                item,
                ..
            } => vec![project, sequence, layer, item],
            Self::TemporalClipEffect {
                project,
                sequence,
                layer,
                item,
                effect,
                parameter,
                ..
            } => vec![project, sequence, layer, item, effect, parameter],
            Self::TemporalApply {
                project,
                sequence,
                apply,
                ..
            }
            | Self::TemporalApplyMask {
                project,
                sequence,
                apply,
                ..
            } => vec![project, sequence, apply],
            Self::TemporalApplyEffect {
                project,
                sequence,
                apply,
                stage,
                effect,
                parameter,
                ..
            } => vec![project, sequence, apply, stage, effect, parameter],
            Self::Item {
                project,
                sequence,
                layer,
                item,
            } => vec![project, sequence, layer, item],
        }
    }
}
