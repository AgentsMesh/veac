mod ast;
mod error;
mod exact;
mod lexer;
mod parser;
mod references;
mod runtime;
mod value;

pub use error::ExpressionError;
pub use exact::ExactNumber;
pub use references::referenced_symbols;
pub use value::{Value, ValueKind, ValueType};

use std::borrow::Borrow;
use std::collections::BTreeMap;

pub type Environment = BTreeMap<String, Value>;

pub(crate) trait ValueLookup {
    fn value(&self, name: &str) -> Option<&Value>;
}

impl<T: Borrow<Value>> ValueLookup for BTreeMap<String, T> {
    fn value(&self, name: &str) -> Option<&Value> {
        self.get(name).map(Borrow::borrow)
    }
}

pub const MAX_EXPRESSION_DEPTH: usize = 64;
pub const MAX_EXPRESSION_NODES: usize = 1_024;
pub(crate) const MAX_TEXT_VALUE_BYTES: usize = 1024 * 1024;

pub fn evaluate(source: &str, env: &Environment) -> Result<Value, ExpressionError> {
    evaluate_with(source, env)
}

pub(crate) fn evaluate_with(source: &str, env: &dyn ValueLookup) -> Result<Value, ExpressionError> {
    let tokens = lexer::lex(source)?;
    let expression = parser::parse(tokens)?;
    runtime::evaluate(&expression, env)
}

#[cfg(test)]
mod tests;
