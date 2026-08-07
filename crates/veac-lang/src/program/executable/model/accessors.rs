use std::collections::BTreeMap;

use super::{BuildInputDeclaration, BuiltProgram, CompiledFunction, ExecutableBuild};
use super::{Diagnostics, MethodRegistry, TypeRegistry};
use crate::program::{SourceIndex, SourceIndexInventory};

macro_rules! accessors {
    ($target:ty) => {
        impl $target {
            pub fn root_module(&self) -> &str {
                &self.root_module
            }

            pub fn sources(&self) -> &BTreeMap<String, String> {
                &self.sources
            }

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
                SourceIndex::build(&self.sources).map(|index| {
                    index.with_build_inputs(crate::program::index::describe_build_inputs(
                        &self.build_inputs,
                        &self.types,
                    ))
                })
            }

            pub fn source_inventory(&self) -> Result<SourceIndexInventory, Diagnostics> {
                self.source_index().map(|index| index.inventory())
            }
        }
    };
}

accessors!(ExecutableBuild);
accessors!(BuiltProgram);
