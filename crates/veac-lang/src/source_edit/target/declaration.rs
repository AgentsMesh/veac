use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{SourceNodeKind, SourceNodeRef};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum DeclarationSite {
    BuildInputDeclaration,
    StructDeclaration,
    StructFieldDeclaration,
    EnumDeclaration,
    EnumVariantDeclaration,
    EnumVariantFieldDeclaration,
    TemporalDeclaration,
    ComponentAnimation { ordinal: u32 },
}

impl DeclarationSite {
    pub fn accepts(self, kind: SourceNodeKind) -> bool {
        matches!(
            (self, kind),
            (Self::BuildInputDeclaration, SourceNodeKind::Input)
                | (Self::StructDeclaration, SourceNodeKind::Struct)
                | (Self::StructFieldDeclaration, SourceNodeKind::StructField)
                | (Self::EnumDeclaration, SourceNodeKind::Enum)
                | (Self::EnumVariantDeclaration, SourceNodeKind::EnumVariant)
                | (
                    Self::EnumVariantFieldDeclaration,
                    SourceNodeKind::EnumVariantField
                )
                | (Self::TemporalDeclaration, SourceNodeKind::Temporal)
                | (Self::ComponentAnimation { .. }, SourceNodeKind::Function)
                | (Self::ComponentAnimation { .. }, SourceNodeKind::Method)
        )
    }

    pub fn accepts_target(self, target: &SourceNodeRef) -> bool {
        self.accepts(target.kind())
    }
}
