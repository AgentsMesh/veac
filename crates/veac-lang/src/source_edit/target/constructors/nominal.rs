use super::super::{SourceNodePath, SourceNodeRef};

impl SourceNodeRef {
    pub fn structure(module: impl Into<String>, structure: impl Into<String>) -> Self {
        Self::new(
            module,
            SourceNodePath::Struct {
                structure: structure.into(),
            },
        )
    }

    pub fn struct_field(
        module: impl Into<String>,
        structure: impl Into<String>,
        field: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::StructField {
                structure: structure.into(),
                field: field.into(),
            },
        )
    }

    pub fn enumeration(module: impl Into<String>, enumeration: impl Into<String>) -> Self {
        Self::new(
            module,
            SourceNodePath::Enum {
                enumeration: enumeration.into(),
            },
        )
    }

    pub fn enum_variant(
        module: impl Into<String>,
        enumeration: impl Into<String>,
        variant: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::EnumVariant {
                enumeration: enumeration.into(),
                variant: variant.into(),
            },
        )
    }

    pub fn enum_variant_field(
        module: impl Into<String>,
        enumeration: impl Into<String>,
        variant: impl Into<String>,
        field: impl Into<String>,
    ) -> Self {
        Self::new(
            module,
            SourceNodePath::EnumVariantField {
                enumeration: enumeration.into(),
                variant: variant.into(),
                field: field.into(),
            },
        )
    }
}
