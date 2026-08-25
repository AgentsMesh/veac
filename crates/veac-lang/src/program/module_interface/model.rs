use super::super::expression::{Effect, FunctionEffect, Stage};
use super::super::CompilerSourceRevision;

mod semantics;
mod types;

pub use semantics::{ModuleCallableSemantics, ModuleParameterDependency, ModuleResultSemantics};
pub use types::{
    ModuleEnumVariantInterface, ModuleFieldInterface, ModuleInterfaceType,
    ModuleTypeDefinitionInterface, ModuleTypeInterface, ModuleTypeName,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleInterface {
    pub source_id: String,
    pub revision: CompilerSourceRevision,
    pub functions: Vec<ModuleFunctionInterface>,
    pub methods: Vec<ModuleMethodInterface>,
    pub types: Vec<ModuleTypeInterface>,
    pub constants: Vec<ModuleConstantInterface>,
    pub domain_capabilities: Vec<ModuleDomainCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleFunctionInterface {
    pub name: String,
    pub parameters: Vec<ModuleParameterInterface>,
    pub return_type: ModuleInterfaceType,
    pub semantics: ModuleCallableSemantics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleMethodInterface {
    pub receiver: ModuleTypeName,
    pub name: String,
    pub parameters: Vec<ModuleParameterInterface>,
    pub return_type: ModuleInterfaceType,
    pub semantics: ModuleCallableSemantics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleParameterInterface {
    pub name: String,
    pub value_type: ModuleInterfaceType,
    pub has_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleConstantInterface {
    pub name: String,
    pub value_type: ModuleInterfaceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDomainCapability {
    pub opcode: u16,
    pub name: String,
}

impl ModuleCallableSemantics {
    pub(crate) fn new(
        effect: Effect,
        contains_local_mutation: bool,
        result: ModuleResultSemantics,
    ) -> Self {
        Self {
            effect,
            contains_local_mutation,
            result,
        }
    }
}

impl ModuleInterfaceType {
    pub(crate) fn function(
        parameters: Vec<Self>,
        return_type: Self,
        effect: FunctionEffect,
    ) -> Self {
        Self::Function {
            parameters,
            return_type: Box::new(return_type),
            effect,
        }
    }
}

impl ModuleResultSemantics {
    pub(crate) fn new(
        shape: Stage,
        leaf: Stage,
        receiver: Option<ModuleParameterDependency>,
        parameters: Vec<ModuleParameterDependency>,
    ) -> Self {
        Self {
            shape,
            leaf,
            receiver,
            parameters,
        }
    }
}
