use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;

pub(super) fn check_active(active: &[String], id: &str) -> Result<(), Vec<Diagnostic>> {
    let path = active.last().map_or(id, String::as_str);
    if let Some(start) = active.iter().position(|value| value == id) {
        let mut chain = active[start..].to_vec();
        chain.push(id.to_owned());
        return Err(vec![Diagnostic::new(
            "PROGRAM_IMPORT_CYCLE",
            path,
            format!("module import cycle: {}", chain.join(" -> ")),
            Span::default(),
        )]);
    }
    if active.len() >= 64 {
        return Err(vec![Diagnostic::new(
            "PROGRAM_IMPORT_DEPTH",
            path,
            "module graph exceeds 64 import levels",
            Span::default(),
        )]);
    }
    Ok(())
}

pub(super) fn source_id(path: &str, message: String) -> Diagnostic {
    Diagnostic::new("PROGRAM_SOURCE_ID", path, message, Span::default())
}

pub(super) fn source_collision(importer: &str, id: &str) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_SOURCE_ID_COLLISION",
        importer,
        format!("source ID `{id}` resolved to different contents"),
        Span::default(),
    )
}

pub(super) fn authority_collision(importer: &str, id: &str) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_SOURCE_AUTHORITY_COLLISION",
        importer,
        format!("source ID `{id}` resolved with conflicting authority"),
        Span::default(),
    )
}

pub(super) fn query_changed(path: &str) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_QUERY_CHANGED",
        path,
        "source dependency state changed during every bounded query attempt",
        Span::default(),
    )
}
