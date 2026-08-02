use std::collections::BTreeSet;

use super::ast::{Expression, ExpressionKind};
use super::ExpressionError;

pub fn referenced_symbols(source: &str) -> Result<BTreeSet<String>, ExpressionError> {
    let expression = super::parser::parse(super::lexer::lex(source)?)?;
    let mut symbols = BTreeSet::new();
    collect(&expression, &mut symbols);
    Ok(symbols)
}

fn collect(expression: &Expression, symbols: &mut BTreeSet<String>) {
    match &expression.kind {
        ExpressionKind::Literal(_) => {}
        ExpressionKind::Symbol(symbol) => {
            symbols.insert(symbol.clone());
        }
        ExpressionKind::Unary { operand, .. } => collect(operand, symbols),
        ExpressionKind::Binary { left, right, .. } => {
            collect(left, symbols);
            collect(right, symbols);
        }
        ExpressionKind::Call { arguments, .. } => {
            for argument in arguments {
                collect(argument, symbols);
            }
        }
    }
}
