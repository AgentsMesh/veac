use super::{error, CoreProgram, ExpressionError};
use crate::program::expression::{MAX_EXPRESSION_DEPTH, MAX_EXPRESSION_NODES};

#[derive(Default)]
pub(super) struct VerifyBudget {
    pub(super) definitions: usize,
    pub(super) values: usize,
}

impl VerifyBudget {
    pub(super) fn enter(
        &mut self,
        program: &CoreProgram,
        depth: usize,
    ) -> Result<(), ExpressionError> {
        if depth > MAX_EXPRESSION_DEPTH {
            return Err(error("nested Core closure depth exceeds its limit", 0..0));
        }
        let Some(definitions) = self
            .definitions
            .checked_add(program.closure_definitions.len())
        else {
            return Err(error("Core closure definition count overflows", 0..0));
        };
        self.definitions = definitions;
        let Some(values) = self.values.checked_add(program.value_count()) else {
            return Err(error("recursive Core value count overflows", 0..0));
        };
        self.values = values;
        if self.definitions > MAX_EXPRESSION_NODES || self.values > MAX_EXPRESSION_NODES {
            return Err(error(
                "recursive Core program exceeds its aggregate limit",
                0..0,
            ));
        }
        Ok(())
    }
}
