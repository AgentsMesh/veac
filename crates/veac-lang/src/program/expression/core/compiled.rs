use std::sync::Arc;

use super::VerifiedCoreProgram;
use super::{CoreDigest, CoreProgram, FunctionId, FunctionRegistry, FunctionSummary};
use crate::program::expression::{FunctionOrigin, FunctionParameter, ValueType};

#[derive(Debug, Clone)]
pub struct CompiledExpression {
    pub(crate) program: VerifiedCoreProgram,
    pub(crate) registry: Arc<FunctionRegistry>,
}

impl CompiledExpression {
    pub(crate) fn new(program: VerifiedCoreProgram, registry: Arc<FunctionRegistry>) -> Self {
        Self { program, registry }
    }

    pub fn result_type(&self) -> &ValueType {
        self.program.core().result_type()
    }

    pub fn core(&self) -> &CoreProgram {
        self.program.core()
    }

    pub(crate) fn registry_arc(&self) -> Arc<FunctionRegistry> {
        Arc::clone(&self.registry)
    }

    pub(crate) fn verified(&self) -> &VerifiedCoreProgram {
        &self.program
    }
}

#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub(super) id: FunctionId,
    pub(super) content_digest: CoreDigest,
    pub(super) summary: FunctionSummary,
    pub(super) name: String,
    pub(super) parameters: Vec<FunctionParameter>,
    pub(super) return_type: ValueType,
    pub(super) origin: Option<FunctionOrigin>,
    pub(crate) body: VerifiedCoreProgram,
}

impl CompiledFunction {
    pub(crate) fn new(
        id: FunctionId,
        content_digest: CoreDigest,
        name: String,
        parameters: Vec<FunctionParameter>,
        return_type: ValueType,
        origin: Option<FunctionOrigin>,
        body: VerifiedCoreProgram,
    ) -> Self {
        let summary = body.core().function_summary();
        Self {
            id,
            content_digest,
            name,
            parameters,
            return_type,
            origin,
            body,
            summary,
        }
    }

    pub fn id(&self) -> FunctionId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn content_digest(&self) -> CoreDigest {
        self.content_digest
    }

    pub fn summary(&self) -> &FunctionSummary {
        &self.summary
    }

    pub fn parameters(&self) -> &[FunctionParameter] {
        &self.parameters
    }

    pub fn return_type(&self) -> &ValueType {
        &self.return_type
    }

    pub fn origin(&self) -> Option<&FunctionOrigin> {
        self.origin.as_ref()
    }

    pub fn body(&self) -> &CoreProgram {
        self.body.core()
    }

    pub(crate) fn verified_body(&self) -> &VerifiedCoreProgram {
        &self.body
    }
}
