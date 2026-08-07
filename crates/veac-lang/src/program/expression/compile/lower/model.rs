use std::collections::BTreeMap;
use std::sync::Arc;

use super::scope::{ClosureContext, Initializer, ScopeBinding};
use super::{FunctionSignatures, SymbolTarget};
use crate::program::expression::{FunctionMap, TypeEnvironment, Value};
use crate::program::TypeRegistry;

pub(super) struct Lowerer<'a> {
    pub(super) symbols: &'a dyn Fn(&str) -> Option<SymbolTarget>,
    pub(super) functions: &'a FunctionMap,
    pub(super) signatures: &'a FunctionSignatures,
    pub(super) methods: &'a crate::program::MethodRegistry,
    pub(super) scopes: Vec<BTreeMap<String, ScopeBinding>>,
    pub(super) closures: Vec<ClosureContext>,
    pub(super) initializers: Vec<Initializer>,
    pub(super) next_local: u32,
    pub(super) next_mutable: u32,
    pub(super) types: Arc<TypeRegistry>,
    pub(super) domain: &'a crate::program::DomainOperationRegistry,
    pub(super) static_values: &'a BTreeMap<String, Arc<Value>>,
    pub(super) provisional_values: &'a TypeEnvironment,
}
