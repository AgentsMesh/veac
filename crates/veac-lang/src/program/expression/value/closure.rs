use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::Value;
use crate::program::expression::core::{
    CoreDigest, FunctionRegistry, FunctionSummary, VerifiedClosureDefinition,
};
use crate::program::expression::ExecutionDefinition;
use crate::program::expression::ValueType;

#[derive(Debug, Clone)]
pub struct ClosureValue {
    definition: Arc<VerifiedClosureDefinition>,
    captures: Arc<[Value]>,
    registry: Arc<FunctionRegistry>,
    provenance: Option<ExecutionDefinition>,
    logical_capture_bytes: usize,
}

impl ClosureValue {
    pub fn value_type(&self) -> &ValueType {
        self.definition.value_type()
    }

    pub fn capture_count(&self) -> usize {
        self.captures.len()
    }

    pub fn capture_types(&self) -> &[ValueType] {
        self.definition.capture_types()
    }

    pub fn summary(&self) -> &FunctionSummary {
        self.definition.summary()
    }

    pub fn definition_digest(&self) -> CoreDigest {
        self.definition.digest()
    }

    pub(crate) fn content_digest(&self) -> CoreDigest {
        let mut digest = Sha256::new();
        digest.update(b"veac.closure-content.v1\0");
        digest.update(self.definition.digest().as_bytes());
        let mut called = self.definition.body().core().called_functions();
        called.sort_unstable();
        called.dedup();
        for id in called {
            digest.update(id.as_bytes());
            let function = self
                .registry
                .get(id)
                .expect("verified closure call target is registered");
            digest.update(function.content_digest().as_bytes());
        }
        CoreDigest::from_digest(digest.finalize().into())
    }

    pub fn logical_capture_bytes(&self) -> usize {
        self.logical_capture_bytes
    }

    pub(crate) fn new(
        definition: Arc<VerifiedClosureDefinition>,
        captures: Vec<Value>,
        registry: Arc<FunctionRegistry>,
        provenance: Option<ExecutionDefinition>,
        logical_capture_bytes: usize,
    ) -> Self {
        Self {
            definition,
            captures: captures.into(),
            registry,
            provenance,
            logical_capture_bytes,
        }
    }

    pub(crate) fn definition(&self) -> &Arc<VerifiedClosureDefinition> {
        &self.definition
    }

    pub(crate) fn captures(&self) -> &[Value] {
        &self.captures
    }

    pub(crate) fn registry(&self) -> &Arc<FunctionRegistry> {
        &self.registry
    }

    pub(crate) fn provenance(&self) -> Option<&ExecutionDefinition> {
        self.provenance.as_ref()
    }
}

impl PartialEq for ClosureValue {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.definition, &other.definition)
            && Arc::ptr_eq(&self.registry, &other.registry)
            && self.captures == other.captures
    }
}

impl Eq for ClosureValue {}
