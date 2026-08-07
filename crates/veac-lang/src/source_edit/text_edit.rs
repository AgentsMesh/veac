use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{budget, SourceEditError, MAX_SOURCE_EDIT_OPERATIONS};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextEdit {
    pub range: TextRange,
    pub replacement: String,
}

#[derive(Clone, Copy)]
pub(crate) struct BorrowedTextEdit<'a> {
    pub(crate) range: TextRange,
    pub(crate) replacement: &'a str,
}

pub fn apply_text_edits(source: &str, edits: &[TextEdit]) -> Result<String, SourceEditError> {
    ensure_edit_count(edits.len())?;
    let edits = edits
        .iter()
        .map(|value| BorrowedTextEdit {
            range: value.range,
            replacement: &value.replacement,
        })
        .collect();
    apply_borrowed_text_edits(source, edits)
}

pub(crate) fn apply_borrowed_text_edits(
    source: &str,
    mut ordered: Vec<BorrowedTextEdit<'_>>,
) -> Result<String, SourceEditError> {
    ensure_edit_count(ordered.len())?;
    ordered.sort_unstable_by_key(|value| (value.range.start, value.range.end));
    for edit in &ordered {
        validate_range(source, edit.range)?;
    }
    for pair in ordered.windows(2) {
        let left = pair[0].range;
        let right = pair[1].range;
        let duplicate_insert =
            left.start == left.end && right.start == right.end && left.start == right.start;
        if left.end > right.start || duplicate_insert {
            return Err(SourceEditError::OverlappingTextEdits);
        }
    }
    let removed = ordered.iter().try_fold(0usize, |bytes, value| {
        budget::checked_add(bytes, value.range.end - value.range.start)
    })?;
    let replacements = ordered.iter().try_fold(0usize, |bytes, value| {
        budget::checked_add(bytes, value.replacement.len())
    })?;
    let plan = budget::text_edit_plan(source.len(), removed, replacements)?;
    let mut result = String::with_capacity(plan.output_bytes);
    let mut cursor = 0;
    for edit in ordered {
        result.push_str(&source[cursor..edit.range.start]);
        result.push_str(edit.replacement);
        cursor = edit.range.end;
    }
    result.push_str(&source[cursor..]);
    debug_assert_eq!(result.len(), plan.output_bytes);
    Ok(result)
}

pub(super) fn ensure_edit_count(count: usize) -> Result<(), SourceEditError> {
    if count > MAX_SOURCE_EDIT_OPERATIONS {
        return Err(SourceEditError::TooManyTextEdits {
            limit: MAX_SOURCE_EDIT_OPERATIONS,
        });
    }
    Ok(())
}

fn validate_range(source: &str, range: TextRange) -> Result<(), SourceEditError> {
    if range.start > range.end || range.end > source.len() {
        return Err(SourceEditError::InvalidTextRange {
            start: range.start,
            end: range.end,
        });
    }
    for offset in [range.start, range.end] {
        if !source.is_char_boundary(offset) {
            return Err(SourceEditError::TextRangeNotUtf8Boundary { offset });
        }
    }
    Ok(())
}
