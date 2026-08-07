use super::Builder;
use crate::program::expression::core::{CoreInput, CoreInputIdentity, InputId};
use crate::program::expression::hir::TypedNode;

impl Builder<'_> {
    pub(in crate::program::expression::compile::core) fn input(
        &mut self,
        name: &str,
        node: &TypedNode,
    ) -> InputId {
        if let Some(id) = self.input_ids.get(name) {
            return *id;
        }
        let id = InputId::new(u32::try_from(self.inputs.len()).expect("input limit fits u32"));
        let type_id = self.types.intern_value(&node.value_type);
        let identity = (self.input_identity)(name);
        let trusted = matches!(identity, CoreInputIdentity::Build(_))
            && node.value_type.is_direct_function()
            && (self.trusted_functions)(name);
        self.inputs.push(CoreInput {
            id,
            name: name.to_owned(),
            identity,
            type_id,
            trusted_function: trusted,
            callable: trusted.then(|| (self.callable_inputs)(name)).flatten(),
            span: node.span.clone(),
        });
        self.input_ids.insert(name.to_owned(), id);
        id
    }
}
