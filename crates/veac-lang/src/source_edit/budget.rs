use super::SourceEditError;

pub const MAX_SOURCE_EDIT_EXPRESSION_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_SOURCE_EDIT_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_SOURCE_EDIT_WORKING_SET_BYTES: usize = 48 * 1024 * 1024;
pub const MAX_SOURCE_EDIT_JSON_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_SOURCE_EDIT_SINGLE_EXPRESSION_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy)]
struct Limits {
    replacement: usize,
    output: usize,
    working: usize,
}

const LIMITS: Limits = Limits {
    replacement: MAX_SOURCE_EDIT_EXPRESSION_PAYLOAD_BYTES,
    output: MAX_SOURCE_EDIT_OUTPUT_BYTES,
    working: MAX_SOURCE_EDIT_WORKING_SET_BYTES,
};

#[derive(Debug)]
pub(super) struct TextEditPlan {
    pub(super) output_bytes: usize,
}

pub(super) fn validate_expression_payload(
    lengths: impl IntoIterator<Item = usize>,
) -> Result<(), SourceEditError> {
    let bytes = lengths.into_iter().try_fold(0usize, checked_add)?;
    if bytes > MAX_SOURCE_EDIT_EXPRESSION_PAYLOAD_BYTES {
        return Err(SourceEditError::ExpressionPayloadTooLarge {
            limit: MAX_SOURCE_EDIT_EXPRESSION_PAYLOAD_BYTES,
        });
    }
    Ok(())
}

pub(super) fn text_edit_plan(
    source_bytes: usize,
    removed_bytes: usize,
    replacement_bytes: usize,
) -> Result<TextEditPlan, SourceEditError> {
    plan_with_limits(source_bytes, removed_bytes, replacement_bytes, LIMITS)
}

fn plan_with_limits(
    source_bytes: usize,
    removed_bytes: usize,
    replacement_bytes: usize,
    limits: Limits,
) -> Result<TextEditPlan, SourceEditError> {
    if replacement_bytes > limits.replacement {
        return Err(SourceEditError::ReplacementPayloadTooLarge {
            limit: limits.replacement,
        });
    }
    let retained = source_bytes
        .checked_sub(removed_bytes)
        .ok_or(SourceEditError::SourceEditSizeOverflow)?;
    let output_bytes = checked_add(retained, replacement_bytes)?;
    if output_bytes > limits.output {
        return Err(SourceEditError::EditedSourceTooLarge {
            limit: limits.output,
        });
    }
    let working = checked_add(source_bytes, replacement_bytes)?;
    let working = checked_add(working, output_bytes)?;
    if working > limits.working {
        return Err(SourceEditError::SourceEditWorkingSetTooLarge {
            limit: limits.working,
        });
    }
    Ok(TextEditPlan { output_bytes })
}

pub(super) fn checked_add(left: usize, right: usize) -> Result<usize, SourceEditError> {
    left.checked_add(right)
        .ok_or(SourceEditError::SourceEditSizeOverflow)
}

#[cfg(test)]
mod tests;
