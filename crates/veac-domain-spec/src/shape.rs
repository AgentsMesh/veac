use veac_lang_model::{PrimitiveType, ValueType};

use super::DomainType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DomainValueShape {
    Primitive(PrimitiveType),
    PrimitiveList(PrimitiveType),
    Domain(DomainType),
    DomainList(DomainType),
}

impl DomainValueShape {
    pub const fn primitive(value: PrimitiveType) -> Self {
        Self::Primitive(value)
    }

    pub const fn primitive_list(value: PrimitiveType) -> Self {
        Self::PrimitiveList(value)
    }

    pub const fn domain(value: DomainType) -> Self {
        Self::Domain(value)
    }

    pub const fn domain_list(value: DomainType) -> Self {
        Self::DomainList(value)
    }

    pub const fn domain_type(self) -> Option<DomainType> {
        match self {
            Self::Primitive(_) | Self::PrimitiveList(_) => None,
            Self::Domain(value) | Self::DomainList(value) => Some(value),
        }
    }

    pub const fn is_single_domain(self) -> bool {
        matches!(self, Self::Domain(_))
    }

    pub fn value_type(self) -> ValueType {
        match self {
            Self::Primitive(value) => ValueType::primitive(value),
            Self::PrimitiveList(value) => ValueType::list(ValueType::primitive(value))
                .expect("one-level primitive list type is valid"),
            Self::Domain(value) => ValueType::domain(value),
            Self::DomainList(value) => ValueType::list(ValueType::domain(value))
                .expect("one-level domain list type is valid"),
        }
    }
}
