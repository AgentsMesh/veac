use std::collections::BTreeMap;
use std::sync::Arc;

use super::{CoreBuildInputId, FunctionMap, TypeEnvironment, Value, ValueType};
use crate::program::{DomainOperationRegistry, MethodRegistry, TypeRegistry};

#[derive(Debug, Clone, Default)]
pub struct ExpressionContext {
    types: Arc<TypeRegistry>,
    functions: Arc<FunctionMap>,
    methods: Arc<MethodRegistry>,
    domain: Arc<DomainOperationRegistry>,
    build_inputs: Arc<BTreeMap<String, BuildInputSlot>>,
    static_values: Arc<BTreeMap<String, Arc<Value>>>,
    provisional_values: Arc<TypeEnvironment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildInputSlot {
    id: CoreBuildInputId,
    name: String,
    value_type: ValueType,
}

impl BuildInputSlot {
    pub fn new(id: CoreBuildInputId, name: String, value_type: ValueType) -> Self {
        Self {
            id,
            name,
            value_type,
        }
    }

    pub fn id(&self) -> CoreBuildInputId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }
}

impl ExpressionContext {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn with_types(mut self, types: Arc<TypeRegistry>) -> Self {
        self.types = types;
        self
    }

    pub fn with_functions(mut self, functions: FunctionMap) -> Self {
        self.functions = Arc::new(functions);
        self
    }

    pub fn with_methods(mut self, methods: MethodRegistry) -> Self {
        self.methods = Arc::new(methods);
        self
    }

    pub fn types(&self) -> &TypeRegistry {
        &self.types
    }

    pub fn functions(&self) -> &FunctionMap {
        &self.functions
    }

    pub fn methods(&self) -> &MethodRegistry {
        &self.methods
    }

    pub fn domain(&self) -> &DomainOperationRegistry {
        &self.domain
    }

    pub fn build_input(&self, name: &str) -> Option<&BuildInputSlot> {
        self.build_inputs.get(name)
    }

    pub fn build_inputs(&self) -> impl Iterator<Item = &BuildInputSlot> {
        self.build_inputs.values()
    }

    pub(crate) fn build_input_bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = (&str, &BuildInputSlot)> {
        self.build_inputs
            .iter()
            .map(|(name, slot)| (name.as_str(), slot))
    }

    pub fn with_build_inputs(mut self, inputs: BTreeMap<String, BuildInputSlot>) -> Self {
        self.build_inputs = Arc::new(inputs);
        self
    }

    pub(crate) fn with_functions_arc(mut self, functions: Arc<FunctionMap>) -> Self {
        self.functions = functions;
        self
    }

    pub(crate) fn functions_arc(&self) -> Arc<FunctionMap> {
        Arc::clone(&self.functions)
    }

    pub(crate) fn with_methods_arc(mut self, methods: Arc<MethodRegistry>) -> Self {
        self.methods = methods;
        self
    }

    pub(crate) fn types_arc(&self) -> Arc<TypeRegistry> {
        Arc::clone(&self.types)
    }

    pub(crate) fn domain_arc(&self) -> Arc<DomainOperationRegistry> {
        Arc::clone(&self.domain)
    }

    pub(crate) fn with_static_values(mut self, values: Arc<BTreeMap<String, Arc<Value>>>) -> Self {
        self.static_values = values;
        self
    }

    pub(crate) fn static_values(&self) -> &BTreeMap<String, Arc<Value>> {
        &self.static_values
    }

    pub(crate) fn with_provisional_values(mut self, values: TypeEnvironment) -> Self {
        self.provisional_values = Arc::new(values);
        self
    }

    pub(crate) fn provisional_values(&self) -> &TypeEnvironment {
        &self.provisional_values
    }
}
