use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IssueCode {
    InvalidSchema,
    InvalidVersion,
    InvalidId,
    DuplicateId,
    InvalidPath,
    InvalidProfile,
    InvalidLocale,
    InvalidLiteral,
    InvalidAction,
    InvalidMatrix,
    MatrixBudgetExceeded,
    UnknownReference,
    BindingConflict,
    InvalidTemplate,
    DeliveryPathConflict,
    SelectorNotUnique,
    SelectorNoMatch,
    DependencyCycle,
    Serialization,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectIssue {
    pub code: IssueCode,
    pub path: String,
    pub message: String,
}

impl ProjectIssue {
    pub(crate) fn new(
        code: IssueCode,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            path: path.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectIssues(pub Vec<ProjectIssue>);

impl ProjectIssues {
    pub fn issues(&self) -> &[ProjectIssue] {
        &self.0
    }

    pub(crate) fn sorted(mut issues: Vec<ProjectIssue>) -> Self {
        issues.sort_by(|left, right| {
            (&left.path, left.code, &left.message).cmp(&(&right.path, right.code, &right.message))
        });
        issues.dedup();
        Self(issues)
    }
}

impl fmt::Display for ProjectIssues {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "project contract has {} issue(s)", self.0.len())
    }
}

impl std::error::Error for ProjectIssues {}
