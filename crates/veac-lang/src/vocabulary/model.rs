use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    CanonicalRole, DomainOpsetSpec, GrammarPosition, IdentifierPolicy, LanguageLayer,
    StandardLibrarySpec, VocabularyCategory, VocabularyValidationError,
};

pub const LANGUAGE_SPEC_SCHEMA: &str = "https://veac.dev/schemas/language-spec";
pub const LANGUAGE_SPEC_SCHEMA_VERSION: u32 = 7;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LanguageSpec {
    #[schemars(extend("const" = LANGUAGE_SPEC_SCHEMA))]
    pub schema: String,
    #[schemars(extend("const" = LANGUAGE_SPEC_SCHEMA_VERSION))]
    pub schema_version: u32,
    #[schemars(extend("const" = "veac"))]
    pub language: String,
    #[schemars(extend("const" = env!("CARGO_PKG_VERSION")))]
    pub language_version: String,
    pub vocabulary: SyntaxVocabulary,
    pub standard_library: StandardLibrarySpec,
    pub domain_opset: DomainOpsetSpec,
    #[schemars(length(min = 1), extend("uniqueItems" = true))]
    pub plugin_effects: Vec<super::PluginEffectSpec>,
}

impl LanguageSpec {
    pub fn current() -> Self {
        Self {
            schema: LANGUAGE_SPEC_SCHEMA.to_owned(),
            schema_version: LANGUAGE_SPEC_SCHEMA_VERSION,
            language: "veac".to_owned(),
            language_version: env!("CARGO_PKG_VERSION").to_owned(),
            vocabulary: SyntaxVocabulary::current(),
            standard_library: StandardLibrarySpec::current(),
            domain_opset: DomainOpsetSpec::current(),
            plugin_effects: super::plugin_effects::current(),
        }
    }

    pub fn validate(&self) -> Result<(), VocabularyValidationError> {
        if self.schema != LANGUAGE_SPEC_SCHEMA
            || self.schema_version != LANGUAGE_SPEC_SCHEMA_VERSION
        {
            return Err(VocabularyValidationError::new(
                "language spec schema identity is not supported",
            ));
        }
        if self.language != "veac" || self.language_version != env!("CARGO_PKG_VERSION") {
            return Err(VocabularyValidationError::new(
                "language identity and version must match this VEAC build",
            ));
        }
        self.vocabulary.validate()?;
        super::plugin_effects::validate(&self.plugin_effects, &self.standard_library)?;
        self.domain_opset
            .validate_with_plugins(&self.plugin_effects)?;
        self.standard_library.validate(&self.domain_opset)
    }
}

impl Default for LanguageSpec {
    fn default() -> Self {
        Self::current()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SyntaxVocabulary {
    #[schemars(length(max = 0))]
    pub lexer_keywords: Vec<String>,
    #[schemars(length(min = 1), extend("uniqueItems" = true))]
    pub entries: Vec<VocabularyEntry>,
}

impl SyntaxVocabulary {
    pub fn current() -> Self {
        super::catalog::current_vocabulary()
    }

    pub fn validate(&self) -> Result<(), VocabularyValidationError> {
        super::validation::validate(self)
    }

    pub fn count(&self, category: VocabularyCategory) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.uses.iter().any(|usage| usage.category == category))
            .count()
    }

    pub fn counts(&self) -> Vec<VocabularyCategoryCount> {
        VocabularyCategory::ALL
            .into_iter()
            .map(|category| VocabularyCategoryCount {
                category,
                count: self.count(category),
            })
            .collect()
    }

    pub fn lookup(&self, spelling: &str) -> Option<&VocabularyEntry> {
        self.entries
            .binary_search_by_key(&spelling, |entry| entry.spelling.as_str())
            .ok()
            .map(|index| &self.entries[index])
    }

    pub fn in_category(
        &self,
        category: VocabularyCategory,
    ) -> impl Iterator<Item = &VocabularyEntry> {
        self.entries
            .iter()
            .filter(move |entry| entry.uses.iter().any(|usage| usage.category == category))
    }
}

impl Default for SyntaxVocabulary {
    fn default() -> Self {
        Self::current()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VocabularyEntry {
    #[schemars(length(min = 1), regex(pattern = r"^\S+$"))]
    pub spelling: String,
    pub identifier_policy: IdentifierPolicy,
    #[schemars(length(min = 1), extend("uniqueItems" = true))]
    pub uses: Vec<SyntaxUse>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SyntaxUse {
    pub category: VocabularyCategory,
    pub layer: LanguageLayer,
    pub position: GrammarPosition,
    pub canonical_role: CanonicalRole,
}

impl SyntaxUse {
    pub(crate) const fn new(
        category: VocabularyCategory,
        position: GrammarPosition,
        canonical_role: CanonicalRole,
    ) -> Self {
        Self {
            category,
            layer: position.layer(),
            position,
            canonical_role,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VocabularyCategoryCount {
    pub category: VocabularyCategory,
    pub count: usize,
}
