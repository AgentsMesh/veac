use super::{Lowerer, SymbolTarget};
use crate::program::expression::ast::Expression;
use crate::program::expression::hir::{TypedCapture, TypedNode, TypedNodeKind};
use crate::program::expression::{ExpressionError, ValueType, MAX_CLOSURE_CAPTURES};

mod capture;
mod checkpoint;
mod model;
mod mutable;
mod static_value;

pub(in crate::program::expression::compile::lower) use model::{
    Checkpoint, ClosureContext, Initializer, ScopeBinding,
};

impl Lowerer<'_> {
    pub(super) fn shadows_static(&self, name: &str) -> bool {
        self.lexical(name).is_some() || (self.symbols)(name).is_some()
    }

    pub(super) fn symbol(
        &mut self,
        name: &str,
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        if let Some(binding) = self.lexical(name) {
            return self.resolve(binding, expression);
        }
        self.reject_self_capture(name, expression)?;
        match (self.symbols)(name) {
            Some(SymbolTarget::Parameter(index, value_type)) => {
                self.resolve(ScopeBinding::parameter(index, value_type, 0), expression)
            }
            Some(SymbolTarget::External(value_type)) => self
                .external_node(name, value_type, false, expression)
                .map(|node| (node.kind, node.value_type)),
            Some(SymbolTarget::TrustedExternal(value_type)) => self
                .external_node(name, value_type, true, expression)
                .map(|node| (node.kind, node.value_type)),
            None => self.static_symbol(name).ok_or_else(|| {
                ExpressionError::new(
                    "EXPRESSION_UNKNOWN_SYMBOL",
                    format!("unknown symbol `{name}`"),
                    expression.span.clone(),
                )
            }),
        }
    }

    pub(super) fn dynamic_callee(
        &mut self,
        name: &str,
        expression: &Expression,
    ) -> Result<Option<TypedNode>, ExpressionError> {
        if let Some(binding) = self.lexical(name) {
            return self.resolved_node(binding, expression).map(Some);
        }
        self.reject_self_capture(name, expression)?;
        match (self.symbols)(name) {
            Some(SymbolTarget::Parameter(index, value_type)) => self
                .resolved_node(ScopeBinding::parameter(index, value_type, 0), expression)
                .map(Some),
            Some(SymbolTarget::External(value_type)) => self
                .external_node(name, value_type, false, expression)
                .map(Some),
            Some(SymbolTarget::TrustedExternal(value_type)) => self
                .external_node(name, value_type, true, expression)
                .map(Some),
            None => Ok(self
                .static_symbol(name)
                .map(|(kind, value_type)| TypedNode {
                    kind,
                    value_type,
                    span: expression.span.clone(),
                })),
        }
    }

    fn external_node(
        &self,
        name: &str,
        value_type: ValueType,
        trusted: bool,
        expression: &Expression,
    ) -> Result<TypedNode, ExpressionError> {
        super::super::known_type::require(&value_type, &self.types).map_err(|error| {
            ExpressionError::new(error.code(), error.message(), expression.span.clone())
        })?;
        let contains_function = value_type.contains_function_in(&self.types) != Some(false);
        if contains_function && (!trusted || !value_type.is_direct_function()) {
            return Err(ExpressionError::new(
                "EXPRESSION_EXTERNAL_FUNCTION_VALUE",
                "external environments cannot inject function values",
                expression.span.clone(),
            ));
        }
        if !self.closures.is_empty() {
            let (code, message) = if contains_function {
                (
                    "EXPRESSION_CLOSURE_FUNCTION_CAPTURE",
                    format!("closure cannot capture function value `{name}`"),
                )
            } else {
                (
                    "EXPRESSION_CLOSURE_EXTERNAL_CAPTURE",
                    format!("closure cannot capture ambient external `{name}`"),
                )
            };
            return Err(ExpressionError::new(code, message, expression.span.clone()));
        }
        Ok(TypedNode {
            kind: TypedNodeKind::External(name.to_owned()),
            value_type,
            span: expression.span.clone(),
        })
    }

    fn lexical(&self, name: &str) -> Option<ScopeBinding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    fn resolve(
        &mut self,
        binding: ScopeBinding,
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let node = self.resolved_node(binding, expression)?;
        Ok((node.kind, node.value_type))
    }

    fn resolved_node(
        &mut self,
        binding: ScopeBinding,
        expression: &Expression,
    ) -> Result<TypedNode, ExpressionError> {
        let mut node = TypedNode {
            kind: binding.kind.node_kind(),
            value_type: binding.value_type.clone(),
            span: expression.span.clone(),
        };
        if binding.owner >= self.closures.len() {
            return Ok(node);
        }
        if matches!(binding.kind, model::BindingKind::Mutable(_)) {
            return Err(ExpressionError::new(
                "EXPRESSION_MUTABLE_CAPTURE",
                "mutable locals cannot be captured by a closure",
                expression.span.clone(),
            ));
        }
        if binding.value_type.contains_function_in(&self.types) != Some(false)
            && self
                .closures
                .iter()
                .skip(binding.owner)
                .any(|closure| !closure.allows_function_captures)
        {
            return Err(ExpressionError::new(
                "EXPRESSION_CLOSURE_FUNCTION_CAPTURE",
                "closures cannot capture function values",
                expression.span.clone(),
            ));
        }
        for closure in self.closures.iter_mut().skip(binding.owner) {
            let index = if let Some(index) = closure.keys.iter().position(|key| *key == binding.key)
            {
                index
            } else {
                if closure.captures.len() >= MAX_CLOSURE_CAPTURES {
                    return Err(ExpressionError::new(
                        "EXPRESSION_CLOSURE_CAPTURE_LIMIT",
                        format!("closure exceeds the {MAX_CLOSURE_CAPTURES} capture limit"),
                        expression.span.clone(),
                    ));
                }
                closure.keys.push(binding.key);
                closure.captures.push(TypedCapture {
                    source: node.clone(),
                });
                closure.captures.len() - 1
            };
            node.kind = TypedNodeKind::Capture(index);
            node.span = expression.span.clone();
        }
        Ok(node)
    }
}
