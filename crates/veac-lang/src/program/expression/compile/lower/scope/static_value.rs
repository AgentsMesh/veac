use super::super::Lowerer;
use crate::program::expression::hir::TypedNodeKind;
use crate::program::expression::ValueType;

impl Lowerer<'_> {
    pub(super) fn static_symbol(&self, name: &str) -> Option<(TypedNodeKind, ValueType)> {
        if let Some(value) = self.static_values.get(name) {
            return Some((
                TypedNodeKind::Literal(value.as_ref().clone()),
                value.value_type(),
            ));
        }
        self.provisional_values
            .get(name)
            .cloned()
            .map(|value_type| (TypedNodeKind::External(name.to_owned()), value_type))
    }

    pub(in crate::program::expression::compile::lower) fn has_static_symbol(
        &self,
        name: &str,
    ) -> bool {
        self.static_values.contains_key(name) || self.provisional_values.contains_key(name)
    }
}
