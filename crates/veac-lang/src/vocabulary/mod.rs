mod catalog;
mod catalog_core;
mod catalog_expression;
mod catalog_modifier_generator;
mod catalog_output;
mod catalog_structure;
mod catalog_text_timeline;
mod catalog_units;
mod catalog_visual;
mod control;
mod domain_contract;
mod enums;
mod expression_name;
mod literal;
mod model;
mod plugin_effects;
mod standard_library;
mod validation;

pub use crate::program::expression::UnitSuffix;
pub(crate) use control::{all_uses as all_control_uses, uses as control_uses};
pub use control::{ControlUse, ControlWord};
pub use domain_contract::{
    DomainEffectSpec, DomainInstructionSpec, DomainOperandAxisSpec, DomainOperandSpec,
    DomainOperationSignature, DomainOperationSpec, DomainOpsetSpec, DomainRuntimeActionSpec,
    DomainTypeSpec, DomainValueShapeSpec,
};
pub use enums::{
    CanonicalRole, GrammarPosition, IdentifierPolicy, LanguageLayer, VocabularyCategory,
};
pub use expression_name::{accepts_expression_name, ExpressionNameKind};
pub use literal::{is_reserved_literal, ReservedLiteral};
pub use model::{
    LanguageSpec, SyntaxUse, SyntaxVocabulary, VocabularyCategoryCount, VocabularyEntry,
    LANGUAGE_SPEC_SCHEMA, LANGUAGE_SPEC_SCHEMA_VERSION,
};
pub use plugin_effects::{
    PluginBackendSpec, PluginDeterminismSpec, PluginEffectSpec, PluginParameterSpec,
    PluginParameterTypeSpec,
};
pub use standard_library::{
    DomainTypeReferenceSpec, StandardLibraryFunction, StandardLibraryMethod, StandardLibrarySpec,
    StandardLibraryType,
};
pub use validation::VocabularyValidationError;

pub fn language_spec() -> LanguageSpec {
    LanguageSpec::current()
}

pub fn language_spec_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schemars::schema_for!(LanguageSpec))
}

#[cfg(test)]
mod tests;
