use std::collections::BTreeSet;

use super::super::ExpressionError;
use crate::program::syntax_document::{SyntaxDocument, SyntaxSlice};

pub(crate) fn referenced_value_symbols_slice(
    document: &SyntaxDocument,
    slice: &SyntaxSlice,
    is_value: &dyn Fn(&str) -> bool,
) -> Result<BTreeSet<String>, ExpressionError> {
    let expression = super::super::cst_adapter::ExpressionSyntax::new(document, slice).parse()?;
    let mut symbols = BTreeSet::new();
    super::collect(&expression, &mut symbols, &BTreeSet::new(), is_value);
    Ok(symbols)
}
