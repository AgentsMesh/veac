mod ast;
mod builtin;
mod collection_operation;
mod compile;
mod context;
mod core;
mod cst_adapter;
mod definition;
mod error;
pub(crate) mod evaluate;
mod exact;
mod execution_budget;
mod hir;
mod index;
mod iteration;
mod lexer;
mod parser;
pub(crate) mod provenance;
mod references;
mod residual;
pub(in crate::program) mod runtime;
mod source_index;
mod temporal_attachment;
mod unit;
mod validation;
mod value;
mod value_type;

pub use builtin::BuiltinFunction;
pub use collection_operation::CollectionOperation;
#[cfg(test)]
pub(crate) use compile::test_support::compile_functions_bounded;
pub use compile::{
    compile_expression, compile_function, compile_functions, compile_temporal_expression,
};
pub(crate) use compile::{
    compile_expression_slice, compile_temporal_slice, lower_function_batch, prepare_function_batch,
    CompiledFunctionBatch, FunctionQueryKey, TypedFunctionBatch,
};
pub use context::{BuildInputSlot, ExpressionContext};
pub use core::{
    ArithmeticOperator, BlockId, ClosureDefinitionId, ComparisonOperator, CompiledExpression,
    CompiledFunction, CoreBlock, CoreBlockParameter, CoreBuildInputId, CoreCallTarget,
    CoreCallableInput, CoreClosureDefinition, CoreDigest, CoreForEach, CoreForEachEffect,
    CoreForEachOrder, CoreForEachProvenance, CoreForEachSlot, CoreForEachSlotId, CoreInput,
    CoreInputDeclarationDigest, CoreInputIdentity, CoreInstruction, CoreInstructionKind,
    CoreLocalSlot, CoreMatchArm, CoreNominalDefinition, CoreProgram, CoreTemporalComposeOperation,
    CoreTemporalInputIdentity, CoreTemporalProjectOperation, CoreTerminator, CoreType,
    CoreTypeEntry, CoreTypeId, CoreTypeTable, CoreUnaryOperator, CoreValueMetadata, DependencyMask,
    Effect, EqualityOperator, FunctionId, FunctionSummary, InputId, LocalSlotId, Stage, ValueId,
    CORE_VERSION,
};
pub use definition::{
    FunctionDefault, FunctionDefinition, FunctionMap, FunctionOrigin, FunctionParameter,
    TypeEnvironment,
};
pub use error::{ExpressionCallFrame, ExpressionError};
pub use evaluate::{compiled as evaluate_compiled, in_context as evaluate_in, source as evaluate};
pub use exact::ExactNumber;
pub(crate) use execution_budget::ExecutionBudget;
pub(in crate::program) use execution_budget::ResidualLedger;
#[cfg(test)]
pub(crate) use execution_budget::{ExecutionLimits, ResourceDelta};
pub(crate) use index::indexed_temporal_attachments;
pub use iteration::{ExpressionLoopFrame, LoopLogicalKey};
pub(crate) use provenance::{DomainOrigin, ExecutionDefinition, ExecutionFrame, ProgramIdentity};
pub use references::referenced_symbols;
#[cfg(test)]
pub(crate) use references::referenced_value_symbols;
pub(crate) use references::referenced_value_symbols_slice;
pub(in crate::program) use residual::{
    residualize_closure_with_ledger, residualize_expression_with_ledger,
};
pub use residual::{
    residualize_expression, ResidualBuildBindings, ResidualInput, ResidualRuntimeValue,
    ResidualValue, ResidualizationError, ResidualizationLimits, ResidualizationRequest,
    ResidualizedExpression,
};
pub(crate) use source_index::indexed_body_sites;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::sync::Arc;
pub(crate) use temporal_attachment::{
    TemporalAttachmentKind, TemporalOwnerKind, TemporalTargetKind,
};
pub use unit::{UnitDimension, UnitSuffix};
pub(crate) use validation::{
    validate_function_body, validate_function_statement, validate_temporal_attachment,
};
pub use value::{
    ClosureValue, DomainValue, EnumValue, ListValue, MapValue, MapValueEntry, PrimitiveType,
    RangeValue, StructValue, TupleValue, Value, ValueConstructionError,
};
pub(crate) use value_type::TypeConstructor;
pub use value_type::{
    FunctionEffect, MapKeyType, ValueType, ValueTypeError, ValueTypeKind, ValueTypeParseError,
    MAX_VALUE_TYPE_ARITY, MAX_VALUE_TYPE_DEPTH,
};
pub type Environment = BTreeMap<String, Value>;

pub(crate) trait ValueLookup {
    fn value(&self, name: &str) -> Option<&Value>;

    fn trusts_function_value(&self, _name: &str) -> bool {
        false
    }

    fn callable_input(&self, name: &str) -> Option<CoreCallableInput> {
        self.trusts_function_value(name)
            .then(|| self.value(name))
            .flatten()
            .and_then(CoreCallableInput::from_value)
    }

    fn build_value(&self, _id: &CoreBuildInputId, name: &str) -> Option<&Value> {
        self.value(name)
    }
}

pub(crate) struct TrustedValueLookup<'a>(&'a dyn ValueLookup);

impl<'a> TrustedValueLookup<'a> {
    pub(crate) fn new(values: &'a dyn ValueLookup) -> Self {
        Self(values)
    }
}

impl ValueLookup for TrustedValueLookup<'_> {
    fn value(&self, name: &str) -> Option<&Value> {
        self.0.value(name)
    }

    fn trusts_function_value(&self, name: &str) -> bool {
        self.0.value(name).is_some()
    }
}

trait LookupValue: Borrow<Value> {
    const TRUSTS_FUNCTIONS: bool;
}

impl LookupValue for Value {
    const TRUSTS_FUNCTIONS: bool = false;
}

impl LookupValue for Arc<Value> {
    const TRUSTS_FUNCTIONS: bool = true;
}

impl<T: LookupValue> ValueLookup for BTreeMap<String, T> {
    fn value(&self, name: &str) -> Option<&Value> {
        self.get(name).map(Borrow::borrow)
    }

    fn trusts_function_value(&self, name: &str) -> bool {
        T::TRUSTS_FUNCTIONS && self.contains_key(name)
    }
}

pub const MAX_EXPRESSION_DEPTH: usize = 64;
pub const MAX_EXPRESSION_NODES: usize = 1_024;
pub const MAX_FUNCTION_CALL_DEPTH: usize = 64;
pub const MAX_FUNCTION_PARAMETERS: usize = 64;
pub const MAX_CALL_ARGUMENTS: usize = 64;
pub const MAX_CLOSURE_CAPTURES: usize = 64;
pub(crate) const MAX_TEXT_VALUE_BYTES: usize = 1024 * 1024;

#[cfg(test)]
pub(crate) use evaluate::{
    lookup as evaluate_lookup_with_functions, lookup_with_budget as evaluate_lookup_with_budget,
};

#[cfg(test)]
mod tests;
