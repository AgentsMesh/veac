use sha2::{Digest, Sha256};

use super::super::super::Stage;
use crate::program::expression::{PrimitiveType, ValueType};
use crate::program::DomainType;
use veac_ir::{ItemId, MaterialId, SequenceId, TemporalParameterId, TemporalType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoreBuildInputId([u8; 32]);

impl CoreBuildInputId {
    pub const fn from_bytes(value: [u8; 32]) -> Self {
        Self(value)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn for_symbol(value: &str) -> Self {
        let mut digest = Sha256::new();
        digest.update(b"veac.core-v10.build-input\0");
        digest.update(value.as_bytes());
        Self(digest.finalize().into())
    }

    pub fn for_declaration(source: &str, name: &str, role: &str, value_type: &str) -> Self {
        let mut digest = Sha256::new();
        digest.update(b"veac.core-v10.declared-build-input\0");
        for value in [source, name, role, value_type] {
            digest.update((value.len() as u64).to_be_bytes());
            digest.update(value.as_bytes());
        }
        Self(digest.finalize().into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoreTemporalInputIdentity {
    SequenceTime {
        sequence_id: SequenceId,
    },
    ClipTime {
        item_id: ItemId,
    },
    SourceTime {
        source_id: MaterialId,
    },
    Frame {
        sequence_id: SequenceId,
    },
    Progress {
        item_id: ItemId,
    },
    Parameter {
        parameter_id: TemporalParameterId,
        value_type: TemporalType,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoreInputIdentity {
    Build(CoreBuildInputId),
    Temporal(CoreTemporalInputIdentity),
}

impl CoreInputIdentity {
    pub(crate) const fn stage(&self) -> Stage {
        match self {
            Self::Build(_) => Stage::Build,
            Self::Temporal(_) => Stage::Temporal,
        }
    }
}

impl CoreTemporalInputIdentity {
    pub const fn temporal_type(&self) -> TemporalType {
        match self {
            Self::SequenceTime { .. } | Self::ClipTime { .. } | Self::SourceTime { .. } => {
                TemporalType::Time
            }
            Self::Frame { .. } => TemporalType::Integer,
            Self::Progress { .. } => TemporalType::Scalar,
            Self::Parameter { value_type, .. } => *value_type,
        }
    }

    pub fn value_type(&self) -> ValueType {
        match self.temporal_type() {
            TemporalType::Boolean => PrimitiveType::Boolean.into(),
            TemporalType::Integer => PrimitiveType::Integer.into(),
            TemporalType::Scalar => PrimitiveType::Scalar.into(),
            TemporalType::Time => PrimitiveType::Time.into(),
            TemporalType::Length => PrimitiveType::Length.into(),
            TemporalType::Angle => PrimitiveType::Angle.into(),
            TemporalType::Color => PrimitiveType::Color.into(),
            TemporalType::Text => PrimitiveType::Text.into(),
            TemporalType::Vec2 => ValueType::domain(DomainType::Vector),
            TemporalType::Point => ValueType::domain(DomainType::Point),
            TemporalType::Rect => ValueType::domain(DomainType::Rect),
        }
    }
}

impl std::fmt::Display for CoreBuildInputId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_hex(formatter, &self.0)
    }
}

fn write_hex(formatter: &mut std::fmt::Formatter<'_>, value: &[u8; 32]) -> std::fmt::Result {
    value
        .iter()
        .try_for_each(|byte| write!(formatter, "{byte:02x}"))
}
