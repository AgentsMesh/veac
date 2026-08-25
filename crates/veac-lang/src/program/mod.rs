//! Deterministic executable modules, expressions, nominal types, and source editing.

mod build_input;
mod compiler_database;
mod dependency_budget;
mod diagnostic;
pub mod domain_system;
mod entry_contract;
mod executable;
pub mod expression;
mod expression_diagnostic;
mod formatter;
mod host_entry;
mod index;
mod lexer;
mod limits;
mod loader;
pub mod method_system;
mod model;
mod module_interface;
mod parser;
mod prepared_source_graph;
mod resolve;
mod source_transaction;
mod source_unit;
mod syntax_document;
mod token;
pub mod type_system;

pub(crate) use build_input::is_material_binding_type;
pub use build_input::{
    build_input_manifest_json_schema, parse_build_input_manifest, BuildInputBinding,
    BuildInputDeclaration, BuildInputManifestV1, BuildInputManifestValue, BuildInputRole,
    BuildInputsError, MaterialInputAuthority, MaterialInputKind, BUILD_INPUT_MANIFEST_SCHEMA,
    BUILD_INPUT_MANIFEST_VERSION, MAX_BUILD_INPUTS, MAX_BUILD_INPUT_MANIFEST_BYTES,
};
pub use compiler_database::{
    CompilerDatabase, CompilerDatabaseLimits, CompilerDatabaseStatistics, CompilerDependencyRoute,
    CompilerDependencySnapshot, CompilerSourceRevision,
};
pub use diagnostic::{Diagnostic, Diagnostics};
pub use domain_system::{
    DomainInstructionKind, DomainOperandContract, DomainOperationContract, DomainOperationExposure,
    DomainOperationId, DomainOperationRegistry, DomainOpsetVersion, DomainRegistryDigest,
    DomainRegistryError, DomainRuntimeAction, DomainType, DomainValueShape, OperandAxis,
    TemporalLoweringOpcode, MAX_DOMAIN_OPERATIONS, MAX_DOMAIN_OPERATION_OPERANDS,
    MAX_DOMAIN_REGISTRY_BYTES,
};
pub use entry_contract::{EntryContract, EntryParameterContract, EntryValueType};
pub use executable::{
    build_path, build_path_with_inputs, build_path_with_root, build_path_with_root_and_inputs,
    build_source, build_source_with_inputs, build_with_loader, prepare_path,
    prepare_path_with_root, prepare_source, prepare_with_loader, BuiltProgram, ExecutableBuild,
};
#[cfg(test)]
pub(crate) use executable::{ClipTemporalProperty, ExecutableTemporalLeaf, ExecutableTemporalSink};
pub use formatter::{format_path, format_source, format_source_with_loader};
pub use host_entry::{
    prepare_host_path, prepare_host_root_path, prepare_host_source, prepare_host_with_loader,
    EvaluatedHostEntry, PreparedHostEntry,
};
pub use index::*;
pub use loader::{
    CompositeSourceLoader, FileSystemLoader, LoadedSource, SourceAuthority, SourceLoader,
};
pub use method_system::{
    MethodBody, MethodDefinition, MethodRegistry, MethodRegistryBuilder, MethodRegistryError,
    MethodSignature, MethodVisibility, MAX_METHODS_PER_TYPE, MAX_METHOD_REGISTRY_BYTES,
    MAX_METHOD_REGISTRY_DEFINITIONS,
};
pub use module_interface::{
    ModuleCallableSemantics, ModuleConstantInterface, ModuleDomainCapability,
    ModuleEnumVariantInterface, ModuleFieldInterface, ModuleFunctionInterface, ModuleInterface,
    ModuleInterfaceType, ModuleMethodInterface, ModuleParameterDependency,
    ModuleParameterInterface, ModuleResultSemantics, ModuleTypeDefinitionInterface,
    ModuleTypeInterface, ModuleTypeName,
};
pub(crate) use parser::{
    validate_fragment as validate_type_declaration_fragment, TypeDeclarationFragmentKind,
};
pub use prepared_source_graph::{PreparedSourceGraph, PreparedSourceGraphRevision};
pub use source_transaction::{
    apply_executable_source_edit_path, apply_executable_source_edit_path_with_inputs,
    apply_executable_source_edit_path_with_root,
    apply_executable_source_edit_path_with_root_and_inputs,
    prepare_executable_source_edit_path_with_root,
    prepare_executable_source_edit_path_with_root_and_database,
    prepare_executable_source_edit_with_loader, reprepare_executable_source_edit_preview,
    ExecutableSourceEditCandidate, ExecutableSourceEditPreview, SourceModuleChange,
    SourceTransactionError,
};
pub use source_unit::{classify_source_unit, SourceUnitKind};
pub use type_system::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionDigest, TypeDefinitionKind, TypeId, TypeRef, TypeRegistry,
    TypeRegistryBuilder, TypeSyntax, TypeSyntaxError, TypeSyntaxKind, VariantIndex,
    MAX_TYPE_DECLARATIONS, MAX_TYPE_MEMBERS, MAX_TYPE_REGISTRY_BYTES,
    MAX_TYPE_REGISTRY_DEFINITIONS,
};
