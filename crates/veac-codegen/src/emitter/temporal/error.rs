use veac_plan::canonical::TemporalBindingId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::emitter) struct TemporalBackendError {
    pub(in crate::emitter) code: &'static str,
    pub(in crate::emitter) binding_id: TemporalBindingId,
    pub(in crate::emitter) message: String,
}

impl TemporalBackendError {
    pub(super) fn new(
        code: &'static str,
        binding_id: &TemporalBindingId,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            binding_id: binding_id.clone(),
            message: message.into(),
        }
    }
}
