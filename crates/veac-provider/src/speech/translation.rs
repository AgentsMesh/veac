use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::TimeRange;

use crate::validation::{language, non_overlapping, text};
use crate::{ProviderResult, TranscriptWord, Validate};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TranslationUnit {
    pub id: String,
    pub range: Option<TimeRange>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TranslationRequest {
    pub source_language: String,
    pub target_language: String,
    pub units: Vec<TranslationUnit>,
    pub preserve_timing: bool,
}

impl Validate for TranslationRequest {
    fn validate(&self) -> ProviderResult<()> {
        language(&self.source_language)?;
        language(&self.target_language)?;
        if self.source_language == self.target_language {
            return crate::validation::invalid("translation languages must differ");
        }
        validate_units(&self.units)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TranslatedUnit {
    pub id: String,
    pub range: Option<TimeRange>,
    pub text: String,
    pub words: Vec<TranscriptWord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TranslationResult {
    pub source_language: String,
    pub target_language: String,
    pub units: Vec<TranslatedUnit>,
}

impl Validate for TranslationResult {
    fn validate(&self) -> ProviderResult<()> {
        language(&self.source_language)?;
        language(&self.target_language)?;
        if self.source_language == self.target_language {
            return crate::validation::invalid("translation result languages must differ");
        }
        let mut previous: Option<&str> = None;
        for unit in &self.units {
            text(&unit.id, "translated unit id")?;
            text(&unit.text, "translated text")?;
            if previous.is_some_and(|value| value >= unit.id.as_str()) {
                return crate::validation::invalid("translated units must be unique and sorted");
            }
            if let Some(value) = unit.range {
                crate::validation::range(value)?;
            }
            for word in &unit.words {
                word.validate()?;
            }
            non_overlapping(&unit.words.iter().map(|word| word.range).collect::<Vec<_>>())?;
            previous = Some(&unit.id);
        }
        Ok(())
    }
}

fn validate_units(units: &[TranslationUnit]) -> ProviderResult<()> {
    let mut previous: Option<&str> = None;
    for unit in units {
        text(&unit.id, "translation unit id")?;
        text(&unit.text, "translation source text")?;
        if previous.is_some_and(|value| value >= unit.id.as_str()) {
            return crate::validation::invalid("translation units must be unique and sorted");
        }
        if let Some(value) = unit.range {
            crate::validation::range(value)?;
        }
        previous = Some(&unit.id);
    }
    Ok(())
}
