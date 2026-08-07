use std::ops::Range;
use std::sync::Arc;

use super::super::metadata::CallableContract;
use super::super::{CoreDigest, CoreTypeId, CoreValueMetadata, FunctionSummary, InputId, Stage};
use crate::program::expression::{ClosureValue, Value, ValueType};

mod identity;
pub use identity::{CoreBuildInputId, CoreInputIdentity, CoreTemporalInputIdentity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreCallableInput {
    pub(crate) summary: FunctionSummary,
    pub(crate) definition_digest: CoreDigest,
    pub(crate) capture_types: Vec<ValueType>,
}

impl CoreCallableInput {
    pub fn summary(&self) -> &FunctionSummary {
        &self.summary
    }

    pub fn definition_digest(&self) -> CoreDigest {
        self.definition_digest
    }

    pub fn capture_types(&self) -> &[ValueType] {
        &self.capture_types
    }

    pub(crate) fn from_value(value: &Value) -> Option<Self> {
        let Value::Closure(value) = value else {
            return None;
        };
        Some(Self {
            summary: value.summary().clone(),
            definition_digest: value.definition_digest(),
            capture_types: value.capture_types().to_vec(),
        })
    }

    pub(crate) fn matches(&self, value: &ClosureValue) -> bool {
        self.summary == *value.summary()
            && self.definition_digest == value.definition_digest()
            && self.capture_types == value.capture_types()
            && self.capture_types.len() == value.capture_count()
    }

    fn metadata(&self, id: InputId) -> CoreValueMetadata {
        let input = CoreValueMetadata::input(Stage::Build, id);
        let captures = vec![input.clone(); self.capture_types.len()];
        let mut output = input;
        output.callable = Some(CallableContract::Closure {
            summary: Arc::new(self.summary.clone()),
            captures: captures.into(),
        });
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreInput {
    pub(crate) id: InputId,
    pub(crate) name: String,
    pub(crate) identity: CoreInputIdentity,
    pub(crate) type_id: CoreTypeId,
    pub(crate) trusted_function: bool,
    pub(crate) callable: Option<CoreCallableInput>,
    pub(crate) span: Range<usize>,
}

impl CoreInput {
    pub fn id(&self) -> InputId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn identity(&self) -> &CoreInputIdentity {
        &self.identity
    }

    pub fn type_id(&self) -> CoreTypeId {
        self.type_id
    }

    pub fn trusts_function_value(&self) -> bool {
        self.trusted_function
    }

    pub fn callable(&self) -> Option<&CoreCallableInput> {
        self.callable.as_ref()
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    pub(crate) fn metadata(&self) -> CoreValueMetadata {
        let stage = self.identity.stage();
        if stage == Stage::Temporal {
            return CoreValueMetadata::temporal_input(self.id);
        }
        self.callable.as_ref().map_or_else(
            || CoreValueMetadata::input(stage, self.id),
            |value| value.metadata(self.id),
        )
    }
}
