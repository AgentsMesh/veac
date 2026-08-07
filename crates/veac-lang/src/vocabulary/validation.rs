use std::fmt;

use super::{
    GrammarPosition, IdentifierPolicy, SyntaxUse, SyntaxVocabulary, VocabularyCategory,
    VocabularyEntry,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VocabularyValidationError {
    message: String,
}

impl VocabularyValidationError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for VocabularyValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for VocabularyValidationError {}

pub(super) fn validate(vocabulary: &SyntaxVocabulary) -> Result<(), VocabularyValidationError> {
    validate_shape(vocabulary)?;
    if vocabulary != &super::catalog::current_vocabulary() {
        return Err(VocabularyValidationError::new(
            "language vocabulary does not match the current descriptors",
        ));
    }
    Ok(())
}

fn validate_shape(vocabulary: &SyntaxVocabulary) -> Result<(), VocabularyValidationError> {
    if !vocabulary.lexer_keywords.is_empty() {
        return Err(VocabularyValidationError::new(
            "VEAC has no lexer-level keywords",
        ));
    }
    for entry in &vocabulary.entries {
        validate_entry(entry)?;
    }
    if vocabulary
        .entries
        .windows(2)
        .any(|pair| pair[0].spelling >= pair[1].spelling)
    {
        return Err(VocabularyValidationError::new(
            "vocabulary entries are not unique and canonically ordered by spelling",
        ));
    }
    for position in GrammarPosition::ALL {
        let inhabited = vocabulary
            .entries
            .iter()
            .flat_map(|entry| &entry.uses)
            .any(|usage| usage.position == *position);
        if !inhabited {
            return Err(VocabularyValidationError::new(format!(
                "published grammar position `{position:?}` has no syntax use"
            )));
        }
    }
    Ok(())
}

fn validate_entry(entry: &VocabularyEntry) -> Result<(), VocabularyValidationError> {
    if entry.spelling.is_empty() || entry.spelling.chars().any(char::is_whitespace) {
        return Err(VocabularyValidationError::new(
            "vocabulary spellings must be non-empty tokens",
        ));
    }
    if entry.uses.is_empty() {
        return Err(VocabularyValidationError::new(format!(
            "vocabulary entry `{}` has no syntax use",
            entry.spelling
        )));
    }
    if entry.uses.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(VocabularyValidationError::new(format!(
            "syntax uses for `{}` are not unique and canonical",
            entry.spelling
        )));
    }
    validate_policy(entry)?;
    for usage in &entry.uses {
        validate_use(entry, usage)?;
    }
    Ok(())
}

fn validate_policy(entry: &VocabularyEntry) -> Result<(), VocabularyValidationError> {
    let word_shaped = crate::name::has_name_shape(&entry.spelling);
    let has_reserved_use = entry
        .uses
        .iter()
        .any(|usage| usage.category == VocabularyCategory::ReservedLiteral);
    let valid = match entry.identifier_policy {
        IdentifierPolicy::Allowed => word_shaped && !has_reserved_use,
        IdentifierPolicy::Reserved => {
            word_shaped
                && has_reserved_use
                && entry
                    .uses
                    .iter()
                    .all(|usage| usage.category == VocabularyCategory::ReservedLiteral)
        }
        IdentifierPolicy::NotApplicable => !word_shaped && !has_reserved_use,
    };
    if !valid {
        return Err(VocabularyValidationError::new(format!(
            "identifier policy for `{}` does not match its lexical shape and uses",
            entry.spelling
        )));
    }
    Ok(())
}

fn validate_use(
    entry: &VocabularyEntry,
    usage: &SyntaxUse,
) -> Result<(), VocabularyValidationError> {
    if !usage.position.is_public() || !usage.layer.is_public() {
        return Err(VocabularyValidationError::new(format!(
            "legacy core-authoring syntax for `{}` is not public",
            entry.spelling
        )));
    }
    if usage.layer != usage.position.layer() {
        return Err(VocabularyValidationError::new(format!(
            "syntax layer for `{}` does not match its grammar position",
            entry.spelling
        )));
    }
    if !usage.canonical_role.accepts(usage.category) {
        return Err(VocabularyValidationError::new(format!(
            "canonical role for `{}` is invalid for its category",
            entry.spelling
        )));
    }
    Ok(())
}
