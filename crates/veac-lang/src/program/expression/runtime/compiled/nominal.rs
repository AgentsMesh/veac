use super::slot::{public, RuntimeValue};
use super::value::contract;
use super::Evaluator;
use crate::program::expression::core::{CoreInstruction, ValueId, VerifiedCoreProgram};
use crate::program::expression::{ExpressionError, Value};
use crate::program::{FieldIndex, TypeId, VariantIndex};

impl Evaluator<'_> {
    pub(super) fn struct_construct(
        &self,
        type_id: TypeId,
        fields: &[ValueId],
        instruction: &CoreInstruction,
        program: &VerifiedCoreProgram,
        values: &[Option<RuntimeValue>],
    ) -> Result<Value, ExpressionError> {
        let fields = fields
            .iter()
            .map(|field| public(values, *field, instruction))
            .collect::<Result<Vec<_>, _>>()?;
        self.execution
            .reserve_nominal(fields.len(), instruction.span.clone())?;
        Value::structure(program.nominal_types(), type_id, fields)
            .map_err(|_| contract("Core struct construction failed", instruction))
    }

    pub(super) fn enum_construct(
        &self,
        type_id: TypeId,
        variant: VariantIndex,
        fields: &[ValueId],
        instruction: &CoreInstruction,
        program: &VerifiedCoreProgram,
        values: &[Option<RuntimeValue>],
    ) -> Result<Value, ExpressionError> {
        let fields = fields
            .iter()
            .map(|field| public(values, *field, instruction))
            .collect::<Result<Vec<_>, _>>()?;
        self.execution
            .reserve_nominal(fields.len(), instruction.span.clone())?;
        Value::variant(program.nominal_types(), type_id, variant, fields)
            .map_err(|_| contract("Core enum construction failed", instruction))
    }

    pub(super) fn struct_project(
        &self,
        structure: ValueId,
        field: FieldIndex,
        instruction: &CoreInstruction,
        values: &[Option<RuntimeValue>],
    ) -> Result<Value, ExpressionError> {
        let Value::Struct(value) = public(values, structure, instruction)? else {
            return Err(contract(
                "Core struct projection receiver is invalid",
                instruction,
            ));
        };
        value
            .fields()
            .get(field.index())
            .cloned()
            .ok_or_else(|| contract("Core struct projection field is invalid", instruction))
    }

    pub(super) fn validate_arguments(
        &self,
        parameters: &[Value],
        captures: &[Value],
        program: &VerifiedCoreProgram,
    ) -> Result<(), ExpressionError> {
        parameters.iter().chain(captures).try_for_each(|value| {
            value
                .validate_nominal_registry(program.nominal_types())
                .map_err(|failure| {
                    ExpressionError::new("EXPRESSION_RUNTIME_CONTRACT", failure.message(), 0..0)
                })?;
            self.domain.validate_argument(value, 0..0)
        })
    }
}
