use crate::Diagnostic;

pub(super) fn precondition_error(id: &str) -> Diagnostic {
    diagnostic(
        "PRECONDITION_FAILED",
        id,
        "/preconditions",
        "edit precondition was not satisfied",
    )
}

pub(super) fn operation_error(id: &str, message: &str) -> Diagnostic {
    diagnostic("EDIT_REJECTED", id, "/operations", message)
}

pub(super) fn diagnostic(code: &str, id: &str, pointer: &str, message: &str) -> Diagnostic {
    Diagnostic {
        code: code.to_owned(),
        object_id: Some(id.to_owned()),
        pointer: pointer.to_owned(),
        message: message.to_owned(),
        suggested_repair: Some(
            "adjust the operation at the indicated JSON pointer and retry".to_owned(),
        ),
    }
}
