use std::collections::BTreeMap;

use super::{BuildInputDeclaration, BuiltProgram, CompiledFunction, ExecutableBuild};
use super::{Diagnostics, MethodRegistry, TypeRegistry};
use crate::program::{PreparedSourceGraph, SourceIndex, SourceIndexInventory};
use crate::source_edit::SourceRevision;

macro_rules! shared_accessors {
    () => {
        pub fn entry_function(&self) -> &CompiledFunction {
            &self.main
        }

        pub fn type_registry(&self) -> &TypeRegistry {
            &self.types
        }

        pub fn method_registry(&self) -> &MethodRegistry {
            &self.methods
        }

        pub fn build_input_declarations(&self) -> &BTreeMap<String, BuildInputDeclaration> {
            &self.build_inputs
        }

        pub fn source_index(&self) -> Result<SourceIndex, Diagnostics> {
            SourceIndex::build(self.source_graph()).map(|index| {
                index.with_build_inputs(crate::program::index::describe_build_inputs(
                    &self.build_inputs,
                    &self.types,
                ))
            })
        }

        pub fn source_inventory(&self) -> Result<SourceIndexInventory, Diagnostics> {
            let index = self.source_index()?;
            let revision = index
                .bound_revision()
                .expect("graph-built source indexes carry complete identity");
            Ok(index
                .inventory(&revision)
                .expect("an index accepts its internally bound revision"))
        }

        pub fn source_revision(&self) -> Result<SourceRevision, Diagnostics> {
            self.source_index().map(|index| {
                index
                    .bound_revision()
                    .expect("graph-built source indexes carry complete identity")
            })
        }
    };
}

impl ExecutableBuild {
    pub fn source_graph(&self) -> &PreparedSourceGraph {
        &self.source_graph
    }

    pub fn root_module(&self) -> &str {
        self.source_graph.root_module()
    }

    pub fn sources(&self) -> &BTreeMap<String, String> {
        self.source_graph.sources()
    }

    shared_accessors!();
}

impl BuiltProgram {
    pub fn source_graph(&self) -> &PreparedSourceGraph {
        &self.source_graph
    }

    pub fn root_module(&self) -> &str {
        self.source_graph.root_module()
    }

    pub fn sources(&self) -> &BTreeMap<String, String> {
        self.source_graph.sources()
    }

    shared_accessors!();
}
