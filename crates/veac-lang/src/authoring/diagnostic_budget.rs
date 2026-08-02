use super::Diagnostic;

pub(crate) const MAX_DIAGNOSTICS: usize = 256;

pub(crate) fn push(diagnostics: &mut Vec<Diagnostic>, diagnostic: Diagnostic) -> bool {
    match diagnostics.len().cmp(&(MAX_DIAGNOSTICS - 1)) {
        std::cmp::Ordering::Less => {
            diagnostics.push(diagnostic);
            false
        }
        std::cmp::Ordering::Equal => {
            diagnostics.push(Diagnostic {
                code: "AUTHORING_DIAGNOSTIC_LIMIT",
                message: "authoring source exceeds the diagnostic budget".to_owned(),
                span: diagnostic.span,
            });
            true
        }
        std::cmp::Ordering::Greater => true,
    }
}

pub(crate) fn exhausted(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.len() >= MAX_DIAGNOSTICS
}
