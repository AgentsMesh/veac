use crate::*;

use crate::edit::operation_error;

#[cfg(test)]
mod tests;

pub(super) fn index<T, I: PartialEq>(
    values: &[T],
    before: Option<&I>,
    after: Option<&I>,
    id_of: impl Fn(&T) -> &I,
    object_id: &str,
) -> Result<usize, Diagnostic> {
    if before.is_some() && after.is_some() {
        return Err(operation_error(
            object_id,
            "insertion accepts exactly one stable neighbor",
        ));
    }
    let Some(neighbor) = before.or(after) else {
        return Ok(values.len());
    };
    let position = values
        .iter()
        .position(|value| id_of(value) == neighbor)
        .ok_or_else(|| operation_error(object_id, "insertion neighbor does not exist"))?;
    Ok(position + usize::from(after.is_some()))
}
