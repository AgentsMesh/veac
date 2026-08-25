use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::diagnostic::Diagnostics;
use super::super::expression::{self, CompiledFunction, FunctionMap};
use super::super::BuildInputDeclaration;
use super::super::{MethodRegistry, PreparedSourceGraph, TypeRegistry};
use super::ExecutableTemporalLeaf;

mod accessors;
mod execution;
mod outputs;

#[derive(Debug, Clone)]
pub struct ExecutableBuild {
    source_graph: PreparedSourceGraph,
    main: Arc<CompiledFunction>,
    functions: Arc<FunctionMap>,
    methods: Arc<MethodRegistry>,
    types: Arc<TypeRegistry>,
    build_inputs: Arc<BTreeMap<String, BuildInputDeclaration>>,
    temporal_leaves: Vec<ExecutableTemporalLeaf>,
}

#[derive(Debug)]
pub struct BuiltProgram {
    source_graph: PreparedSourceGraph,
    main: Arc<CompiledFunction>,
    methods: Arc<MethodRegistry>,
    types: Arc<TypeRegistry>,
    build_inputs: Arc<BTreeMap<String, BuildInputDeclaration>>,
    #[allow(dead_code)]
    graph: expression::runtime::domain_graph::FrozenDomainGraph,
    envelope: veac_ir::ProjectEnvelope,
}

pub(super) struct ExecutableRegistries {
    functions: Arc<FunctionMap>,
    methods: Arc<MethodRegistry>,
    types: Arc<TypeRegistry>,
    build_inputs: Arc<BTreeMap<String, BuildInputDeclaration>>,
}

impl ExecutableRegistries {
    pub(super) fn new(
        functions: Arc<FunctionMap>,
        methods: Arc<MethodRegistry>,
        types: Arc<TypeRegistry>,
        build_inputs: Arc<BTreeMap<String, BuildInputDeclaration>>,
    ) -> Self {
        Self {
            functions,
            methods,
            types,
            build_inputs,
        }
    }
}

impl ExecutableBuild {
    pub(super) fn new(
        source_graph: PreparedSourceGraph,
        main: Arc<CompiledFunction>,
        registries: ExecutableRegistries,
        temporal_leaves: Vec<ExecutableTemporalLeaf>,
    ) -> Self {
        Self {
            source_graph,
            main,
            functions: registries.functions,
            methods: registries.methods,
            types: registries.types,
            build_inputs: registries.build_inputs,
            temporal_leaves,
        }
    }
}

impl BuiltProgram {
    pub fn envelope(&self) -> &veac_ir::ProjectEnvelope {
        &self.envelope
    }

    #[allow(dead_code)]
    pub(in crate::program) fn graph(
        &self,
    ) -> &expression::runtime::domain_graph::FrozenDomainGraph {
        &self.graph
    }
}

#[cfg(test)]
mod test_api;
