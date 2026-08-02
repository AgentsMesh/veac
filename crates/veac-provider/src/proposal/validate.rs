use super::ProviderEditProposal;
use crate::{ProviderError, ProviderErrorKind, ProviderResult, Validate};

mod source;

impl Validate for ProviderEditProposal {
    fn validate(&self) -> ProviderResult<()> {
        self.request_hash.validate()?;
        self.response_hash.validate()?;
        source::validate(self)?;
        if self.project_revision != self.batch.base_revision
            || !self.batch.atomic
            || self.batch.operations.is_empty()
            || self.batch.operations.len() != self.evidence.len()
        {
            return invalid("provider edit proposal structure is inconsistent");
        }
        for (index, (evidence, operation)) in
            self.evidence.iter().zip(&self.batch.operations).enumerate()
        {
            if usize::try_from(evidence.operation_index()).ok() != Some(index)
                || !evidence.matches_operation(operation)
                || !super::evidence_validate::matches(evidence, self, operation)
            {
                return invalid("provider proposal evidence does not match its operation");
            }
        }
        veac_ir::canonical_edit_batch_json(&self.batch).map_err(|error| {
            ProviderError::with_source(
                ProviderErrorKind::InvalidContract,
                "provider edit proposal contains an invalid edit batch",
                error,
            )
        })?;
        Ok(())
    }
}

fn invalid<T>(message: &str) -> ProviderResult<T> {
    Err(ProviderError::new(
        ProviderErrorKind::InvalidContract,
        message,
    ))
}
