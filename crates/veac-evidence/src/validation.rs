use std::collections::BTreeSet;

use crate::{EvidenceSuiteV1, RegionSpace, EVIDENCE_SUITE_SCHEMA_VERSION};

mod assertions;
mod references;

const MAX_ITEMS: usize = 512;

#[derive(Debug, Clone)]
pub struct ValidatedSuite(EvidenceSuiteV1);

impl ValidatedSuite {
    pub fn as_suite(&self) -> &EvidenceSuiteV1 {
        &self.0
    }

    pub fn into_suite(self) -> EvidenceSuiteV1 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub issues: Vec<ValidationIssue>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "evidence suite has {} validation error(s)",
            self.issues.len()
        )
    }
}

impl std::error::Error for ValidationError {}

pub fn validate(suite: EvidenceSuiteV1) -> Result<ValidatedSuite, ValidationError> {
    let mut issues = Vec::new();
    if suite.schema_version != EVIDENCE_SUITE_SCHEMA_VERSION {
        issue(&mut issues, "schema_version", "unsupported schema version");
    }
    check_id(&mut issues, "id", &suite.id);
    check_limit(&mut issues, "sources", suite.sources.len());
    check_limit(&mut issues, "samples", suite.samples.len());
    check_limit(&mut issues, "regions", suite.regions.len());
    check_limit(&mut issues, "assertions", suite.assertions.len());
    if suite.sources.is_empty() || suite.assertions.is_empty() {
        issue(
            &mut issues,
            "suite",
            "sources and assertions must be non-empty",
        );
    }
    unique_ids(
        &mut issues,
        "sources",
        suite.sources.iter().map(|value| value.id.as_str()),
    );
    unique_ids(
        &mut issues,
        "samples",
        suite.samples.iter().map(|value| value.id.as_str()),
    );
    unique_ids(
        &mut issues,
        "regions",
        suite.regions.iter().map(|value| value.id.as_str()),
    );
    unique_ids(
        &mut issues,
        "assertions",
        suite.assertions.iter().map(|value| value.id()),
    );
    validate_regions(&suite, &mut issues);
    references::validate(&suite, &mut issues);
    assertions::validate(&suite, &mut issues);
    if issues.is_empty() {
        Ok(ValidatedSuite(suite))
    } else {
        Err(ValidationError { issues })
    }
}

fn unique_ids<'a>(
    issues: &mut Vec<ValidationIssue>,
    path: &str,
    values: impl Iterator<Item = &'a str>,
) {
    let mut found = BTreeSet::new();
    for (index, value) in values.enumerate() {
        check_id(issues, &format!("{path}[{index}].id"), value);
        if !found.insert(value) {
            issue(issues, &format!("{path}[{index}].id"), "duplicate id");
        }
    }
}

fn validate_regions(suite: &EvidenceSuiteV1, issues: &mut Vec<ValidationIssue>) {
    for (index, region) in suite.regions.iter().enumerate() {
        let valid = match region.space {
            RegionSpace::Normalized {
                x,
                y,
                width,
                height,
            } => {
                [x, y, width, height].iter().all(|value| value.is_finite())
                    && x >= 0.0
                    && y >= 0.0
                    && width > 0.0
                    && height > 0.0
                    && x + width <= 1.0
                    && y + height <= 1.0
            }
            RegionSpace::Pixels { width, height, .. } => width > 0 && height > 0,
        };
        if !valid {
            issue(
                issues,
                &format!("regions[{index}].space"),
                "invalid region geometry",
            );
        }
    }
}

pub(super) fn check_id(issues: &mut Vec<ValidationIssue>, path: &str, value: &str) {
    let valid = !value.is_empty()
        && value.len() <= 96
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit() && index > 0
                || matches!(byte, b'_' | b'-' | b'.') && index > 0
        });
    if !valid {
        issue(
            issues,
            path,
            "id must use lowercase ASCII, digits, '.', '_' or '-'",
        );
    }
}

pub(super) fn issue(issues: &mut Vec<ValidationIssue>, path: &str, message: &str) {
    issues.push(ValidationIssue {
        path: path.to_owned(),
        message: message.to_owned(),
    });
}

fn check_limit(issues: &mut Vec<ValidationIssue>, path: &str, count: usize) {
    if count > MAX_ITEMS {
        issue(issues, path, "collection exceeds the 512 item budget");
    }
}
