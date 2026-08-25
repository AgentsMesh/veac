use std::collections::BTreeSet;

use super::ast::{Block, Expression, ExpressionKind};
use super::ExpressionError;

mod syntax;
pub(crate) use syntax::referenced_value_symbols_slice;
#[cfg(test)]
mod test_support;
#[cfg(test)]
pub(crate) use test_support::referenced_value_symbols;

pub fn referenced_symbols(source: &str) -> Result<BTreeSet<String>, ExpressionError> {
    referenced(source, &|_| false)
}

fn referenced(
    source: &str,
    direct_callee_is_value: &dyn Fn(&str) -> bool,
) -> Result<BTreeSet<String>, ExpressionError> {
    let expression = super::parser::parse(super::lexer::lex(source)?)?;
    let mut symbols = BTreeSet::new();
    collect(
        &expression,
        &mut symbols,
        &BTreeSet::new(),
        direct_callee_is_value,
    );
    Ok(symbols)
}

fn collect(
    expression: &Expression,
    symbols: &mut BTreeSet<String>,
    locals: &BTreeSet<String>,
    direct_callee_is_value: &dyn Fn(&str) -> bool,
) {
    match &expression.kind {
        ExpressionKind::Literal(_) => {}
        ExpressionKind::Symbol(symbol) => {
            if !locals.contains(symbol) {
                symbols.insert(symbol.clone());
            }
        }
        ExpressionKind::Unary { operand, .. } => {
            collect(operand, symbols, locals, direct_callee_is_value)
        }
        ExpressionKind::Binary { left, right, .. } => {
            collect(left, symbols, locals, direct_callee_is_value);
            collect(right, symbols, locals, direct_callee_is_value);
        }
        ExpressionKind::Range { start, end, step } => {
            collect(start, symbols, locals, direct_callee_is_value);
            collect(end, symbols, locals, direct_callee_is_value);
            if let Some(step) = step {
                collect(step, symbols, locals, direct_callee_is_value);
            }
        }
        ExpressionKind::Closure {
            parameters, body, ..
        } => {
            let mut nested = locals.clone();
            nested.extend(parameters.iter().map(|value| value.name.clone()));
            collect_block(body, symbols, &nested, direct_callee_is_value);
        }
        ExpressionKind::For {
            binding,
            iterable,
            body,
            ..
        } => {
            collect(iterable, symbols, locals, direct_callee_is_value);
            let mut nested = locals.clone();
            nested.insert(binding.clone());
            collect_block(body, symbols, &nested, direct_callee_is_value);
        }
        ExpressionKind::Call { callee, arguments } => {
            if let Some(path) = super::ast::static_path(callee) {
                collect_path(&path, symbols, locals, direct_callee_is_value, true);
            } else {
                collect(callee, symbols, locals, direct_callee_is_value);
            }
            for argument in arguments {
                collect(&argument.value, symbols, locals, direct_callee_is_value);
            }
        }
        ExpressionKind::FieldProject { receiver, .. } => {
            if let Some(path) = super::ast::static_path(expression) {
                collect_path(&path, symbols, locals, direct_callee_is_value, false);
            } else {
                collect(receiver, symbols, locals, direct_callee_is_value);
            }
        }
        ExpressionKind::NominalConstruct { fields, .. } => {
            for field in fields {
                collect(&field.value, symbols, locals, direct_callee_is_value);
            }
        }
        ExpressionKind::Match { scrutinee, arms } => {
            collect(scrutinee, symbols, locals, direct_callee_is_value);
            for arm in arms {
                let mut nested = locals.clone();
                if let super::ast::MatchPattern::Variant { fields, .. } = &arm.pattern {
                    nested.extend(fields.iter().map(|field| field.binding.clone()));
                }
                collect(&arm.body, symbols, &nested, direct_callee_is_value);
            }
        }
        ExpressionKind::TemporalAttach(value) => {
            for argument in &value.arguments {
                collect(argument, symbols, locals, direct_callee_is_value);
            }
            let mut nested = locals.clone();
            nested.extend([
                "sequence_time".to_owned(),
                "clip_time".to_owned(),
                "frame".to_owned(),
                "progress".to_owned(),
                "source_time".to_owned(),
            ]);
            collect_block(&value.body, symbols, &nested, direct_callee_is_value);
        }
        ExpressionKind::List(values) | ExpressionKind::Tuple(values) => {
            for value in values {
                collect(value, symbols, locals, direct_callee_is_value);
            }
        }
        ExpressionKind::Map(entries) => {
            for entry in entries {
                collect(&entry.key, symbols, locals, direct_callee_is_value);
                collect(&entry.value, symbols, locals, direct_callee_is_value);
            }
        }
        ExpressionKind::Block(block) => {
            collect_block(block, symbols, locals, direct_callee_is_value)
        }
        ExpressionKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect(condition, symbols, locals, direct_callee_is_value);
            collect_block(then_branch, symbols, locals, direct_callee_is_value);
            collect_block(else_branch, symbols, locals, direct_callee_is_value);
        }
    }
}

fn collect_path(
    path: &[super::ast::PathSegment],
    symbols: &mut BTreeSet<String>,
    locals: &BTreeSet<String>,
    is_value: &dyn Fn(&str) -> bool,
    callee: bool,
) {
    if locals.contains(&path[0].name) {
        return;
    }
    for count in (1..=path.len()).rev() {
        let prefix = super::ast::join_path(&path[..count]);
        if is_value(&prefix) {
            symbols.insert(prefix);
            return;
        }
    }
    if !callee {
        symbols.insert(super::ast::join_path(path));
    }
}

fn collect_block(
    block: &Block,
    symbols: &mut BTreeSet<String>,
    outer: &BTreeSet<String>,
    direct_callee_is_value: &dyn Fn(&str) -> bool,
) {
    let mut locals = outer.clone();
    for statement in &block.statements {
        use super::ast::Statement;
        match statement {
            Statement::Let(binding) => {
                collect(&binding.value, symbols, &locals, direct_callee_is_value);
                locals.insert(binding.name.clone());
            }
            Statement::Var(binding) => {
                collect(&binding.value, symbols, &locals, direct_callee_is_value);
                locals.insert(binding.name.clone());
            }
            Statement::Set(assignment) => {
                collect(&assignment.value, symbols, &locals, direct_callee_is_value);
            }
        }
    }
    collect(&block.result, symbols, &locals, direct_callee_is_value);
}
