use crate::source_edit::MAX_SOURCE_EDIT_SINGLE_FRAGMENT_BYTES;

pub(super) fn validate(source: &str) -> Result<(), &'static str> {
    if source.trim().is_empty() || source.len() > MAX_SOURCE_EDIT_SINGLE_FRAGMENT_BYTES {
        return Err("source must contain 1..65536 bytes");
    }
    if source.contains('\0') {
        return Err("source must not contain NUL");
    }
    Ok(())
}
