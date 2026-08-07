use super::{require_binding_type, Initializer, Lowerer, ScopeBinding};
use crate::program::expression::ast::{LetBinding, MutableBinding, Statement};
use crate::program::expression::hir::{
    LocalId, MutableLocalId, TypedBinding, TypedMutableAssignment, TypedMutableBinding,
    TypedStatement,
};
use crate::program::expression::ExpressionError;

impl Lowerer<'_> {
    pub(super) fn statement(
        &mut self,
        statement: &Statement,
    ) -> Result<TypedStatement, ExpressionError> {
        match statement {
            Statement::Let(binding) => self.immutable_binding(binding),
            Statement::Var(binding) => self.mutable_binding(binding),
            Statement::Set(assignment) => {
                let (id, expected) =
                    self.mutable_target(&assignment.name, assignment.name_span.clone())?;
                let value = self.lower_context(&assignment.value, Some(&expected))?;
                require_binding_type(&assignment.name, &value, &expected)?;
                Ok(TypedStatement::Set(TypedMutableAssignment { id, value }))
            }
        }
    }

    fn immutable_binding(
        &mut self,
        binding: &LetBinding,
    ) -> Result<TypedStatement, ExpressionError> {
        let value =
            self.binding_value(&binding.name, binding.annotation.as_ref(), &binding.value)?;
        let id = LocalId::new(self.next_local);
        self.next_local = next(self.next_local);
        self.insert_binding(
            &binding.name,
            ScopeBinding::local(id, value.value_type.clone(), self.closures.len()),
        );
        Ok(TypedStatement::Let(TypedBinding { id, value }))
    }

    fn mutable_binding(
        &mut self,
        binding: &MutableBinding,
    ) -> Result<TypedStatement, ExpressionError> {
        let value =
            self.binding_value(&binding.name, binding.annotation.as_ref(), &binding.value)?;
        if value.value_type.contains_function_in(&self.types) != Some(false) {
            return Err(ExpressionError::new(
                "EXPRESSION_MUTABLE_FUNCTION",
                "mutable locals cannot contain function values",
                binding.name_span.clone(),
            ));
        }
        let id = MutableLocalId::new(self.next_mutable);
        self.next_mutable = next(self.next_mutable);
        self.insert_binding(
            &binding.name,
            ScopeBinding::mutable(id, value.value_type.clone(), self.closures.len()),
        );
        Ok(TypedStatement::Var(TypedMutableBinding { id, value }))
    }

    fn binding_value(
        &mut self,
        name: &str,
        annotation: Option<&crate::program::expression::ast::TypeAnnotation>,
        expression: &crate::program::expression::ast::Expression,
    ) -> Result<crate::program::expression::hir::TypedNode, ExpressionError> {
        let annotation = annotation
            .map(|value| self.resolve_annotation(value))
            .transpose()?;
        self.initializers.push(Initializer {
            name: name.to_owned(),
            owner: self.closures.len(),
        });
        let value = self.lower_context(expression, annotation.as_ref());
        self.initializers.pop();
        let value = value?;
        if let Some(expected) = annotation.as_ref() {
            require_binding_type(name, &value, expected)?;
        }
        Ok(value)
    }

    fn insert_binding(&mut self, name: &str, binding: ScopeBinding) {
        self.scopes
            .last_mut()
            .expect("block scope is active")
            .insert(name.to_owned(), binding);
    }
}

fn next(value: u32) -> u32 {
    value
        .checked_add(1)
        .expect("parser node limit bounds local IDs")
}
