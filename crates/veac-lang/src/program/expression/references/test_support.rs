use std::collections::BTreeSet;

use super::super::ExpressionError;

pub(crate) fn referenced_value_symbols(
    source: &str,
    is_value: &dyn Fn(&str) -> bool,
) -> Result<BTreeSet<String>, ExpressionError> {
    super::referenced(source, is_value)
}
