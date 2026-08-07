use std::ops::Range;
use std::sync::Arc;

use crate::program::TypeDefinition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreNominalDefinition {
    pub(crate) definition: Arc<TypeDefinition>,
    pub(crate) span: Range<usize>,
}

impl CoreNominalDefinition {
    pub(crate) fn new(definition: Arc<TypeDefinition>, span: Range<usize>) -> Self {
        Self { definition, span }
    }

    pub fn definition(&self) -> &TypeDefinition {
        &self.definition
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    pub(crate) fn definition_handle(&self) -> Arc<TypeDefinition> {
        Arc::clone(&self.definition)
    }
}
