use super::super::super::expression::{FunctionEffect, MapKeyType, PrimitiveType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleInterfaceType {
    Primitive(PrimitiveType),
    Domain {
        name: String,
        opcode: u16,
    },
    Named(ModuleTypeName),
    List(Box<ModuleInterfaceType>),
    Range(PrimitiveType),
    Map {
        key: MapKeyType,
        value: Box<ModuleInterfaceType>,
    },
    Tuple(Vec<ModuleInterfaceType>),
    Function {
        parameters: Vec<ModuleInterfaceType>,
        return_type: Box<ModuleInterfaceType>,
        effect: FunctionEffect,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleTypeName {
    pub source_id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleTypeInterface {
    pub name: String,
    pub definition: ModuleTypeDefinitionInterface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleTypeDefinitionInterface {
    Struct {
        fields: Vec<ModuleFieldInterface>,
    },
    Enum {
        variants: Vec<ModuleEnumVariantInterface>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleFieldInterface {
    pub name: String,
    pub value_type: ModuleInterfaceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEnumVariantInterface {
    pub name: String,
    pub fields: Vec<ModuleFieldInterface>,
}
