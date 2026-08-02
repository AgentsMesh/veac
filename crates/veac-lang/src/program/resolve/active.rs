use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;

pub(super) fn check(
    active: &[String],
    importer: &str,
    id: &str,
    span: Span,
) -> Result<(), Diagnostic> {
    if let Some(start) = active.iter().position(|value| value == id) {
        let mut chain = active[start..].to_vec();
        chain.push(id.to_owned());
        return Err(Diagnostic::new(
            "PROGRAM_IMPORT_CYCLE",
            importer,
            format!("module import cycle: {}", chain.join(" -> ")),
            span,
        ));
    }
    if active.len() >= 64 {
        return Err(Diagnostic::new(
            "PROGRAM_IMPORT_DEPTH",
            importer,
            "module graph exceeds 64 import levels",
            span,
        ));
    }
    Ok(())
}
