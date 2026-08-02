mod replacements;

use crate::program::diagnostic::Diagnostic;

pub(super) use replacements::Replacements;

pub(super) const MAX_EXPANDED_BYTES: usize = 32 * 1024 * 1024;
pub(super) const MAX_REPLACEMENTS: usize = 16_384;
const SOURCE_MESSAGE: &str = "expanded source exceeds 32 MiB";

pub(super) fn ensure(
    path: &str,
    length: usize,
    limit: usize,
    message: &'static str,
) -> Result<(), Diagnostic> {
    if length <= limit {
        Ok(())
    } else {
        Err(error(path, message))
    }
}

pub(super) fn ensure_source(path: &str, length: usize, limit: usize) -> Result<(), Diagnostic> {
    ensure(path, length, limit, SOURCE_MESSAGE)
}

pub(super) fn ensure_replacement_available(
    path: &str,
    replacement_count: usize,
) -> Result<(), Diagnostic> {
    if replacement_count < MAX_REPLACEMENTS {
        Ok(())
    } else {
        Err(error(path, "expansion replacement budget exceeded"))
    }
}

pub(super) fn checked_add(
    path: &str,
    left: usize,
    right: usize,
    message: &'static str,
) -> Result<usize, Diagnostic> {
    left.checked_add(right).ok_or_else(|| error(path, message))
}

pub(super) fn checked_source_add(
    path: &str,
    left: usize,
    right: usize,
) -> Result<usize, Diagnostic> {
    checked_add(path, left, right, SOURCE_MESSAGE)
}

pub(super) fn consume(path: &str, remaining: usize, amount: usize) -> Result<usize, Diagnostic> {
    remaining
        .checked_sub(amount)
        .ok_or_else(|| size_error(path))
}

pub(super) fn size_error(path: &str) -> Diagnostic {
    error(path, SOURCE_MESSAGE)
}

fn error(path: &str, message: &'static str) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_EXPANSION_BUDGET",
        path,
        message,
        crate::authoring::Span::default(),
    )
}

#[cfg(test)]
mod tests;
