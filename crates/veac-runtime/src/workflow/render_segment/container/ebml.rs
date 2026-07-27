use super::super::media_error;
use crate::workflow::WorkflowResult;

const EBML_MAGIC: [u8; 4] = [0x1a, 0x45, 0xdf, 0xa3];
const DOC_TYPE_ID: u64 = 0x4282;

#[derive(Clone, Copy)]
pub(super) enum DocType {
    Matroska,
    Webm,
}

impl DocType {
    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Matroska => b"matroska",
            Self::Webm => b"webm",
        }
    }
}

pub(super) fn validate(prefix: &[u8], expected: DocType) -> WorkflowResult<()> {
    if doc_type(prefix) != Some(expected.bytes()) {
        return Err(media_error(
            "render-segment EBML document type differs from its container",
        ));
    }
    Ok(())
}

fn doc_type(bytes: &[u8]) -> Option<&[u8]> {
    if !bytes.starts_with(&EBML_MAGIC) {
        return None;
    }
    let (header_size, size_len) = vint(bytes.get(4..)?, false)?;
    let mut cursor = 4_usize.checked_add(size_len)?;
    let end = cursor.checked_add(usize::try_from(header_size).ok()?)?;
    if end > bytes.len() {
        return None;
    }
    while cursor < end {
        let (id, id_len) = vint(bytes.get(cursor..)?, true)?;
        cursor = cursor.checked_add(id_len)?;
        let (size, size_len) = vint(bytes.get(cursor..)?, false)?;
        cursor = cursor.checked_add(size_len)?;
        let item_end = cursor.checked_add(usize::try_from(size).ok()?)?;
        if item_end > end {
            return None;
        }
        if id == DOC_TYPE_ID {
            return bytes.get(cursor..item_end);
        }
        cursor = item_end;
    }
    None
}

fn vint(bytes: &[u8], preserve_marker: bool) -> Option<(u64, usize)> {
    let first = *bytes.first()?;
    let length = usize::try_from(first.leading_zeros())
        .ok()?
        .checked_add(1)?;
    if length > 8 || bytes.len() < length {
        return None;
    }
    let mask = 0xff_u8.checked_shr(u32::try_from(length).ok()?)?;
    let mut value = u64::from(if preserve_marker { first } else { first & mask });
    for byte in &bytes[1..length] {
        value = value.checked_mul(256)?.checked_add(u64::from(*byte))?;
    }
    if !preserve_marker {
        let unknown = (1_u128 << (7 * length)) - 1;
        if u128::from(value) == unknown {
            return None;
        }
    }
    Some((value, length))
}

#[cfg(test)]
#[path = "ebml/tests.rs"]
mod tests;
