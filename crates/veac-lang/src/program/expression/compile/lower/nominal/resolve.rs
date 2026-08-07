use crate::program::expression::ast::PathSegment;
use crate::program::{TypeDefinitionKind, TypeRef};

use super::super::Lowerer;

#[derive(Clone)]
pub(in crate::program::expression::compile::lower) struct ResolvedType {
    pub reference: TypeRef,
    pub kind: TypeDefinitionKind,
    pub consumed: usize,
}

impl Lowerer<'_> {
    pub(in crate::program::expression::compile::lower) fn resolve_nominal_prefix(
        &self,
        path: &[PathSegment],
    ) -> Option<ResolvedType> {
        for consumed in (1..=path.len()).rev() {
            let name = crate::program::expression::ast::join_path(&path[..consumed]);
            let Some(reference) = self.types.resolve(&name).cloned() else {
                continue;
            };
            let definition = self
                .types
                .definition(reference.id())
                .expect("verified type name resolves to a definition");
            return Some(ResolvedType {
                reference,
                kind: definition.kind().clone(),
                consumed,
            });
        }
        None
    }
}
