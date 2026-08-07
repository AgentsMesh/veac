//! Deterministic executable modules, expressions, nominal types, and source editing.

mod build_input;
mod dependency_budget;
mod diagnostic;
pub mod domain_system;
mod executable;
pub mod expression;
mod expression_diagnostic;
mod formatter;
mod index;
mod lexer;
mod limits;
mod loader;
pub mod method_system;
mod model;
mod parser;
mod resolve;
mod source_transaction;
mod source_unit;
mod token;
pub mod type_system;

pub use build_input::{
    build_input_manifest_json_schema, parse_build_input_manifest, BuildInputBinding,
    BuildInputDeclaration, BuildInputManifestV1, BuildInputManifestValue, BuildInputRole,
    BuildInputsError, BUILD_INPUT_MANIFEST_SCHEMA, BUILD_INPUT_MANIFEST_VERSION, MAX_BUILD_INPUTS,
    MAX_BUILD_INPUT_MANIFEST_BYTES,
};
pub use diagnostic::{Diagnostic, Diagnostics};
pub use domain_system::{
    DomainInstructionKind, DomainOperandContract, DomainOperationContract, DomainOperationExposure,
    DomainOperationId, DomainOperationRegistry, DomainOpsetVersion, DomainRegistryDigest,
    DomainRegistryError, DomainRuntimeAction, DomainType, DomainValueShape, OperandAxis,
    TemporalLoweringOpcode, MAX_DOMAIN_OPERATIONS, MAX_DOMAIN_OPERATION_OPERANDS,
    MAX_DOMAIN_REGISTRY_BYTES,
};
pub use executable::{
    build_path, build_path_with_inputs, build_path_with_root, build_path_with_root_and_inputs,
    build_source, build_source_with_inputs, build_with_loader, prepare_path,
    prepare_path_with_root, prepare_source, prepare_with_loader, BuiltProgram, ExecutableBuild,
};
#[cfg(test)]
pub(crate) use executable::{ClipTemporalProperty, ExecutableTemporalLeaf, ExecutableTemporalSink};
pub use formatter::{format_path, format_source, format_source_with_loader};
pub use index::*;
pub use loader::{FileSystemLoader, LoadedSource, SourceLoader};
pub use method_system::{
    MethodBody, MethodDefinition, MethodRegistry, MethodRegistryBuilder, MethodRegistryError,
    MethodSignature, MethodVisibility, MAX_METHODS_PER_TYPE, MAX_METHOD_REGISTRY_BYTES,
    MAX_METHOD_REGISTRY_DEFINITIONS,
};
pub(crate) use parser::{
    validate_fragment as validate_type_declaration_fragment, TypeDeclarationFragmentKind,
};
pub use source_transaction::{
    apply_executable_source_edit_path, apply_executable_source_edit_path_with_inputs,
    apply_executable_source_edit_path_with_root,
    apply_executable_source_edit_path_with_root_and_inputs,
    prepare_executable_source_edit_path_with_root, ExecutableSourceEditCandidate,
    ExecutableSourceEditPreview, SourceModuleChange, SourceTransactionError,
};
pub use source_unit::{classify_source_unit, SourceUnitKind};
pub use type_system::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionDigest, TypeDefinitionKind, TypeId, TypeRef, TypeRegistry,
    TypeRegistryBuilder, TypeSyntax, TypeSyntaxError, TypeSyntaxKind, VariantIndex,
    MAX_TYPE_DECLARATIONS, MAX_TYPE_MEMBERS, MAX_TYPE_REGISTRY_BYTES,
    MAX_TYPE_REGISTRY_DEFINITIONS,
};
