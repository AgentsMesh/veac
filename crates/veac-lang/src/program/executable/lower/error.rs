use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorClass {
    Lower,
    Ir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::program::executable) struct ExecutableLowerError {
    class: ErrorClass,
    reason: &'static str,
    message: String,
}

impl ExecutableLowerError {
    pub(in crate::program::executable) fn lower(
        reason: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            class: ErrorClass::Lower,
            reason,
            message: message.into(),
        }
    }

    pub(in crate::program::executable) fn ir(error: impl std::fmt::Display) -> Self {
        Self {
            class: ErrorClass::Ir,
            reason: "EXECUTABLE_LOWER_IR_VALIDATION",
            message: format!("the executable graph did not produce valid canonical IR: {error}"),
        }
    }

    pub(in crate::program::executable) const fn diagnostic_code(&self) -> &'static str {
        match self.class {
            ErrorClass::Lower => "PROGRAM_EXECUTABLE_LOWER",
            ErrorClass::Ir => "PROGRAM_EXECUTABLE_IR",
        }
    }

    pub(in crate::program::executable) const fn reason_code(&self) -> &'static str {
        self.reason
    }
}

impl fmt::Display for ExecutableLowerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ExecutableLowerError {}
