use std::sync::Arc;

use veac_plan::canonical::TemporalBindingId;

use super::error::TemporalBackendError;

pub(super) const MAX_EXPRESSION_BYTES: usize = 96 * 1024;
pub(super) const MAX_COMPILED_BYTES: usize = 512 * 1024;
pub(super) const MAX_TEXT_CHOICES: usize = 256;

pub(super) struct Budget<'a> {
    binding_id: &'a TemporalBindingId,
    bytes: usize,
}

impl<'a> Budget<'a> {
    pub(super) fn new(binding_id: &'a TemporalBindingId) -> Self {
        Self {
            binding_id,
            bytes: 0,
        }
    }

    pub(super) fn expression(&mut self, parts: &[&str]) -> Result<Arc<str>, TemporalBackendError> {
        let length = parts.iter().try_fold(0usize, |sum, part| {
            sum.checked_add(part.len()).ok_or_else(|| self.exceeded())
        })?;
        let total = self
            .bytes
            .checked_add(length)
            .ok_or_else(|| self.exceeded())?;
        if length > MAX_EXPRESSION_BYTES || total > MAX_COMPILED_BYTES {
            return Err(self.exceeded());
        }
        self.bytes = total;
        let mut result = String::with_capacity(length);
        for part in parts {
            result.push_str(part);
        }
        Ok(Arc::from(result))
    }

    pub(super) fn literal(&mut self, value: String) -> Result<Arc<str>, TemporalBackendError> {
        self.expression(&[&value])
    }

    pub(super) fn unsupported(&self, message: impl Into<String>) -> TemporalBackendError {
        TemporalBackendError::new("TEMPORAL_BACKEND_UNSUPPORTED", self.binding_id, message)
    }

    pub(super) const fn binding_id(&self) -> &TemporalBindingId {
        self.binding_id
    }

    pub(super) fn contract(&self, message: impl Into<String>) -> TemporalBackendError {
        TemporalBackendError::new("TEMPORAL_BACKEND_CONTRACT", self.binding_id, message)
    }

    fn exceeded(&self) -> TemporalBackendError {
        TemporalBackendError::new(
            "TEMPORAL_EXPRESSION_BUDGET",
            self.binding_id,
            "compiled temporal expression exceeds the FFmpeg backend budget",
        )
    }
}
