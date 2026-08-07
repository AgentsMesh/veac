use std::sync::Arc;

use super::slot::public_ref;
use super::value::contract;
use super::Evaluator;
use crate::program::expression::core::{
    ClosureDefinitionId, CoreInstruction, ValueId, VerifiedCoreProgram,
};
use crate::program::expression::{
    ClosureValue, ExecutionFrame, ExpressionError, Value, MAX_FUNCTION_CALL_DEPTH,
};
use crate::program::DomainOperationRegistry;

impl Evaluator<'_> {
    pub(super) fn closure(
        &self,
        definition_id: ClosureDefinitionId,
        capture_ids: &[ValueId],
        instruction: &CoreInstruction,
        program: &VerifiedCoreProgram,
        values: &[Option<super::slot::RuntimeValue>],
    ) -> Result<Value, ExpressionError> {
        let definition = program
            .closure(definition_id)
            .cloned()
            .ok_or_else(|| contract("verified closure definition is unavailable", instruction))?;
        if definition.id() != definition_id {
            return Err(contract(
                "verified closure definition ID is inconsistent",
                instruction,
            ));
        }
        let captures = capture_ids
            .iter()
            .map(|id| public_ref(values, *id, instruction))
            .collect::<Result<Vec<_>, _>>()?;
        let logical_capture_bytes = self
            .execution
            .reserve_closure(captures.len(), instruction.span.clone())?;
        let captures = captures.into_iter().cloned().collect::<Vec<_>>();
        let provenance = self.frames.last().map(|frame| {
            frame
                .definition()
                .closure(definition.digest(), instruction.span.clone())
        });
        Ok(Value::Closure(Arc::new(ClosureValue::new(
            definition,
            captures,
            Arc::clone(&self.registry),
            provenance,
            logical_capture_bytes,
        ))))
    }

    pub(super) fn invoke(
        &mut self,
        callee: Value,
        arguments: Vec<Value>,
        instruction: &CoreInstruction,
        call_depth: usize,
    ) -> Result<Value, ExpressionError> {
        if call_depth >= MAX_FUNCTION_CALL_DEPTH {
            return Err(ExpressionError::new(
                "EXPRESSION_CALL_DEPTH_LIMIT",
                format!("function calls exceed the {MAX_FUNCTION_CALL_DEPTH} depth limit"),
                instruction.span.clone(),
            ));
        }
        let Value::Closure(closure) = callee else {
            return Err(contract(
                "Invoke callee is not a verified closure",
                instruction,
            ));
        };
        if arguments.len() != closure.definition().parameter_types().len()
            || arguments
                .iter()
                .zip(closure.definition().parameter_types())
                .any(|(value, expected)| &value.value_type() != expected)
        {
            return Err(contract(
                "Invoke runtime argument signature mismatch",
                instruction,
            ));
        }
        super::validate_core_domain_identity(
            closure.definition().body().core(),
            DomainOperationRegistry::shared(),
        )?;
        let caller_registry = std::mem::replace(&mut self.registry, Arc::clone(closure.registry()));
        let frame = closure.provenance().cloned().map(|definition| {
            ExecutionFrame::new(definition, self.origin(instruction.span.clone()))
        });
        let result = self.program(
            closure.definition().body(),
            &arguments,
            closure.captures(),
            call_depth + 1,
            None,
            frame,
        );
        self.registry = caller_registry;
        result
    }
}
