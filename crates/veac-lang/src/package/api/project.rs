use super::*;
use crate::package::PackageIdentity;
use crate::program::{
    expression::{Effect, FunctionEffect, MapKeyType as CompilerMapKey, PrimitiveType, Stage},
    ModuleCallableSemantics, ModuleFieldInterface, ModuleInterface, ModuleInterfaceType,
    ModuleParameterDependency, ModuleTypeDefinitionInterface, ModuleTypeName,
};

mod order;

use order::key;

pub fn from_module_interface(
    package: PackageIdentity,
    interface: &ModuleInterface,
) -> ApiMetadataV1 {
    let mut exports = interface
        .functions
        .iter()
        .map(|value| ApiExport::Function {
            name: value.name.clone(),
            parameters: parameters(&value.parameters),
            return_type: value_type(&value.return_type),
            semantics: semantics(&value.semantics),
        })
        .chain(interface.methods.iter().map(|value| ApiExport::Method {
            receiver: type_name(&value.receiver),
            name: value.name.clone(),
            parameters: parameters(&value.parameters),
            return_type: value_type(&value.return_type),
            semantics: semantics(&value.semantics),
        }))
        .chain(interface.types.iter().map(|value| ApiExport::Type {
            name: value.name.clone(),
            definition: type_definition(&value.definition),
        }))
        .chain(interface.constants.iter().map(|value| ApiExport::Constant {
            name: value.name.clone(),
            value_type: value_type(&value.value_type),
        }))
        .collect::<Vec<_>>();
    exports.sort_by_key(key);
    let capabilities = interface
        .domain_capabilities
        .iter()
        .map(|value| ApiDomainCapability {
            opcode: value.opcode,
            name: value.name.clone(),
        })
        .collect();
    ApiMetadataV1::new(package, exports).with_domain_capabilities(capabilities)
}

fn parameters(values: &[crate::program::ModuleParameterInterface]) -> Vec<ApiFunctionParameter> {
    values
        .iter()
        .map(|value| ApiFunctionParameter {
            name: value.name.clone(),
            value_type: value_type(&value.value_type),
            has_default: value.has_default,
        })
        .collect()
}

fn semantics(value: &ModuleCallableSemantics) -> ApiCallableSemantics {
    ApiCallableSemantics {
        effect: match value.effect {
            Effect::Pure => ApiSemanticEffect::Pure,
            Effect::LocalMutation => ApiSemanticEffect::LocalMutation,
            Effect::GraphEmit => ApiSemanticEffect::GraphEmit,
        },
        contains_local_mutation: value.contains_local_mutation,
        result: ApiResultSemantics {
            shape: stage(value.result.shape),
            leaf: stage(value.result.leaf),
            receiver: value.result.receiver.map(dependency),
            parameters: value
                .result
                .parameters
                .iter()
                .copied()
                .map(dependency)
                .collect(),
        },
    }
}

fn dependency(value: ModuleParameterDependency) -> ApiParameterDependency {
    ApiParameterDependency {
        shape_from_shape: value.shape_from_shape,
        shape_from_leaf: value.shape_from_leaf,
        leaf_from_shape: value.leaf_from_shape,
        leaf_from_leaf: value.leaf_from_leaf,
    }
}

fn stage(value: Stage) -> ApiStage {
    match value {
        Stage::Const => ApiStage::Const,
        Stage::Build => ApiStage::Build,
        Stage::Temporal => ApiStage::Temporal,
    }
}

fn value_type(value: &ModuleInterfaceType) -> ApiType {
    match value {
        ModuleInterfaceType::Primitive(value) => ApiType::Primitive {
            name: primitive(*value),
        },
        ModuleInterfaceType::Domain { name, opcode } => ApiType::Domain {
            name: name.clone(),
            opcode: *opcode,
        },
        ModuleInterfaceType::Named(value) => ApiType::Named {
            name: type_name(value),
        },
        ModuleInterfaceType::List(value) => ApiType::List {
            element: Box::new(value_type(value)),
        },
        ModuleInterfaceType::Range(value) => ApiType::Range {
            element: primitive(*value),
        },
        ModuleInterfaceType::Map { key, value } => ApiType::Map {
            key: match key {
                CompilerMapKey::Text => MapKeyType::Text,
                CompilerMapKey::Identifier => MapKeyType::Identifier,
            },
            value: Box::new(value_type(value)),
        },
        ModuleInterfaceType::Tuple(values) => ApiType::Tuple {
            elements: values.iter().map(value_type).collect(),
        },
        ModuleInterfaceType::Function {
            parameters,
            return_type,
            effect,
        } => ApiType::Function {
            parameters: parameters.iter().map(value_type).collect(),
            return_type: Box::new(value_type(return_type)),
            effect: match effect {
                FunctionEffect::Pure => ApiEffect::Pure,
                FunctionEffect::Local => ApiEffect::Local,
                FunctionEffect::Emit => ApiEffect::Emit,
                FunctionEffect::Any => ApiEffect::Any,
            },
        },
    }
}

fn primitive(value: PrimitiveType) -> ApiPrimitive {
    match value {
        PrimitiveType::Integer => ApiPrimitive::Int,
        PrimitiveType::Scalar => ApiPrimitive::Scalar,
        PrimitiveType::Time => ApiPrimitive::Time,
        PrimitiveType::Length => ApiPrimitive::Length,
        PrimitiveType::Percent => ApiPrimitive::Percent,
        PrimitiveType::Angle => ApiPrimitive::Angle,
        PrimitiveType::Text => ApiPrimitive::Text,
        PrimitiveType::Color => ApiPrimitive::Color,
        PrimitiveType::Boolean => ApiPrimitive::Bool,
        PrimitiveType::Identifier => ApiPrimitive::Identifier,
    }
}

fn type_name(value: &ModuleTypeName) -> ApiTypeName {
    ApiTypeName {
        module: value.source_id.clone(),
        name: value.name.clone(),
    }
}

fn type_definition(value: &ModuleTypeDefinitionInterface) -> ApiTypeDefinition {
    match value {
        ModuleTypeDefinitionInterface::Struct { fields: values } => ApiTypeDefinition::Struct {
            fields: fields(values),
        },
        ModuleTypeDefinitionInterface::Enum { variants } => ApiTypeDefinition::Enum {
            variants: variants
                .iter()
                .map(|value| ApiEnumVariant {
                    name: value.name.clone(),
                    fields: fields(&value.fields),
                })
                .collect(),
        },
    }
}

fn fields(values: &[ModuleFieldInterface]) -> Vec<ApiField> {
    values
        .iter()
        .map(|value| ApiField {
            name: value.name.clone(),
            value_type: value_type(&value.value_type),
        })
        .collect()
}
