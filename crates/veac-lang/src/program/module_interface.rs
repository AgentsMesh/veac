//! Immutable compiler-derived interface for an exported module.

mod capabilities;
mod model;
mod project;
mod retained;

use super::expression::ExecutionBudget;
use super::{CompilerDatabase, Diagnostics, LoadedSource, SourceLoader};
use std::sync::Arc;

pub use model::{
    ModuleCallableSemantics, ModuleConstantInterface, ModuleDomainCapability,
    ModuleEnumVariantInterface, ModuleFieldInterface, ModuleFunctionInterface, ModuleInterface,
    ModuleInterfaceType, ModuleMethodInterface, ModuleParameterDependency,
    ModuleParameterInterface, ModuleResultSemantics, ModuleTypeDefinitionInterface,
    ModuleTypeInterface, ModuleTypeName,
};

impl CompilerDatabase {
    pub fn module_interface(
        &self,
        root: LoadedSource,
        loader: &dyn SourceLoader,
    ) -> Result<ModuleInterface, Diagnostics> {
        self.module_interface_shared(root, loader)
            .map(|value| value.as_ref().clone())
    }

    pub fn module_interface_shared(
        &self,
        root: LoadedSource,
        loader: &dyn SourceLoader,
    ) -> Result<Arc<ModuleInterface>, Diagnostics> {
        let (graph, admission) = self
            .interface_source_graph(root, loader)
            .map_err(Diagnostics)?;
        let epoch = self.query_epoch();
        let key = graph.interface_key();
        if let Some(value) = self.cached_interface(&epoch, &key) {
            self.consume_invalidation(&admission);
            return Ok(value);
        }
        let root = graph.root();
        let revision = self.source_revision(&root.id, &root.source);
        let source_id = root.id.clone();
        let scope =
            super::resolve::module_interface_scope(root, &graph, &ExecutionBudget::default(), self)
                .map_err(Diagnostics)?;
        let value = Arc::new(project::scope(source_id.clone(), revision, &scope)?);
        let value = self.cache_interface(&epoch, key, retained::bytes(&value), value);
        self.consume_invalidation(&admission);
        Ok(value)
    }
}
