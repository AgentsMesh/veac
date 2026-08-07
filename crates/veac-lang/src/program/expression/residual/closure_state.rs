use super::ResidualRuntimeValue;
use crate::program::expression::ClosureDefinitionId;

#[derive(Clone)]
pub(super) struct ResidualClosure {
    pub(super) definition: ClosureDefinitionId,
    pub(super) captures: Vec<ResidualRuntimeValue>,
}
