use super::super::definitions::Definitions;
use super::super::error;
use crate::program::expression::core::{CoreInstruction, CoreValueMetadata, ValueId};
use crate::program::expression::{CollectionOperation, ExpressionError};

pub(super) fn verify_callback(
    operation: CollectionOperation,
    callable: ValueId,
    argument_count: usize,
    definitions: &Definitions,
    instruction: &CoreInstruction,
) -> Result<(), ExpressionError> {
    let arguments = vec![CoreValueMetadata::constant(); argument_count];
    let evidence = definitions
        .metadata(callable)
        .known_callable_effect(&arguments);
    super::super::effect::verify_collection(operation, evidence)
        .map_err(|violation| error(violation.message(), instruction.span.clone()))
}
