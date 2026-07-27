use crate::emitter::CodegenErrorKind;

#[derive(Debug)]
pub(super) struct Failure {
    pub code: &'static str,
    pub kind: CodegenErrorKind,
    pub message: String,
    pub object_id: Option<String>,
}

impl Failure {
    pub fn invalid(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            kind: CodegenErrorKind::InvalidPlan,
            message: message.into(),
            object_id: None,
        }
    }

    pub fn unsupported(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            kind: CodegenErrorKind::UnsupportedCaptionFeature,
            message: message.into(),
            object_id: None,
        }
    }

    pub fn at(mut self, object_id: impl ToString) -> Self {
        self.object_id = Some(object_id.to_string());
        self
    }
}
