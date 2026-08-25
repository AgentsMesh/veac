use sha2::{Digest, Sha256};

use super::super::FunctionOrigin;
use crate::program::expression::{FunctionId, ValueType, CORE_VERSION};

const DEFAULT_ID_DOMAIN: &[u8] = b"veac.parameter-default.v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionParameter {
    pub name: String,
    pub value_type: ValueType,
    default: Option<FunctionDefault>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDefault {
    source: String,
    origin: Option<FunctionOrigin>,
    thunk: Option<FunctionId>,
}

impl FunctionParameter {
    pub fn new(name: impl Into<String>, value_type: ValueType) -> Self {
        Self {
            name: name.into(),
            value_type,
            default: None,
        }
    }

    pub fn with_default(
        mut self,
        source: impl Into<String>,
        origin: Option<FunctionOrigin>,
    ) -> Self {
        self.default = Some(FunctionDefault {
            source: source.into(),
            origin,
            thunk: None,
        });
        self
    }

    pub fn has_default(&self) -> bool {
        self.default.is_some()
    }

    pub fn default(&self) -> Option<&FunctionDefault> {
        self.default.as_ref()
    }

    pub(crate) fn bind_default(&mut self, owner: FunctionId, slot: usize) {
        if let Some(value) = &mut self.default {
            value.thunk = Some(default_id(owner, slot, &value.source));
        }
    }
}

impl FunctionDefault {
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn origin(&self) -> Option<&FunctionOrigin> {
        self.origin.as_ref()
    }

    pub(crate) fn thunk(&self) -> FunctionId {
        self.thunk.expect("compiled signature binds default thunk")
    }
}

pub(crate) fn default_id(owner: FunctionId, slot: usize, source: &str) -> FunctionId {
    let mut digest = Sha256::new();
    digest.update(DEFAULT_ID_DOMAIN);
    digest.update(CORE_VERSION.to_be_bytes());
    digest.update(owner.as_bytes());
    digest.update((slot as u64).to_be_bytes());
    digest.update((source.len() as u64).to_be_bytes());
    digest.update(source.as_bytes());
    FunctionId::from_bytes(digest.finalize().into())
}
