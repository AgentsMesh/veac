mod cue;
mod style;

use std::{collections::BTreeSet, fmt};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CaptionEnvelope, CURRENT_SCHEMA_VERSION, SCHEMA_ID};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidationIssue {
    pub path: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationErrors(pub Vec<ValidationIssue>);

impl fmt::Display for ValidationErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, issue) in self.0.iter().enumerate() {
            if index > 0 {
                formatter.write_str("; ")?;
            }
            write!(
                formatter,
                "{} [{}]: {}",
                issue.path, issue.code, issue.message
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

pub fn validate(value: &CaptionEnvelope) -> Result<(), ValidationErrors> {
    let mut issues = Vec::new();
    if value.schema != SCHEMA_ID {
        issue(
            &mut issues,
            "$.schema",
            "SCHEMA",
            "unsupported schema identifier",
        );
    }
    if value.schema_version != CURRENT_SCHEMA_VERSION {
        issue(
            &mut issues,
            "$.schema_version",
            "VERSION",
            "unsupported schema version",
        );
    }
    let document = &value.document;
    if document.timescale == 0 {
        issue(
            &mut issues,
            "$.document.timescale",
            "TIMESCALE",
            "must be greater than zero",
        );
    }
    optional_text(
        &mut issues,
        "$.document.language",
        document.language.as_deref(),
    );
    settings(&mut issues, "$.document.settings", &document.settings);
    style::validate_styles(document, &mut issues);
    cue::validate_cues(document, &mut issues);
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ValidationErrors(issues))
    }
}

pub(crate) fn issue(issues: &mut Vec<ValidationIssue>, path: &str, code: &str, message: &str) {
    issues.push(ValidationIssue {
        path: path.to_owned(),
        code: code.to_owned(),
        message: message.to_owned(),
    });
}

pub(crate) fn optional_text(issues: &mut Vec<ValidationIssue>, path: &str, value: Option<&str>) {
    if value.is_some_and(|text| text.trim().is_empty()) {
        issue(issues, path, "EMPTY", "must be absent or nonempty");
    }
}

pub(crate) fn settings(
    issues: &mut Vec<ValidationIssue>,
    path: &str,
    values: &std::collections::BTreeMap<String, String>,
) {
    if values
        .iter()
        .any(|(key, value)| key.trim().is_empty() || value.trim().is_empty())
    {
        issue(issues, path, "SETTING", "keys and values must be nonempty");
    }
}

pub(crate) fn duplicates<'a>(mut values: impl Iterator<Item = &'a str>) -> bool {
    let mut seen = BTreeSet::new();
    values.any(|value| !seen.insert(value))
}
