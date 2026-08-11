use std::collections::BTreeSet;

use crate::{IssueCode, ProjectIssue, ProjectTarget};

use super::{check_id, ManifestIndex};

pub(crate) fn check_profiles(
    index: &ManifestIndex<'_>,
    target: &ProjectTarget,
    base: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    let mut seen = BTreeSet::new();
    for (position, profile) in target.profiles.iter().enumerate() {
        let path = format!("{base}.profiles[{position}]");
        check_id(profile.as_str(), &path, issues);
        if !seen.insert(profile) {
            duplicate(&path, profile.as_str(), issues);
        }
        if !index.profiles.contains(profile.as_str()) {
            issues.push(ProjectIssue::new(
                IssueCode::UnknownReference,
                path,
                format!("unknown profile {:?}", profile.as_str()),
            ));
        }
    }
}

pub(crate) fn check_axes(target: &ProjectTarget, base: &str, issues: &mut Vec<ProjectIssue>) {
    if target.axes.len() > 16 {
        invalid(
            &format!("{base}.axes"),
            "target cannot define more than 16 axes",
            issues,
        );
    }
    let mut axes = BTreeSet::new();
    for (position, axis) in target.axes.iter().enumerate() {
        let path = format!("{base}.axes[{position}]");
        check_id(axis.id.as_str(), &format!("{path}.id"), issues);
        if !axes.insert(&axis.id) {
            duplicate(&format!("{path}.id"), axis.id.as_str(), issues);
        }
        if axis.values.is_empty() {
            invalid(
                &format!("{path}.values"),
                "axis must contain values",
                issues,
            );
        }
        let mut values = BTreeSet::new();
        for (value_position, value) in axis.values.iter().enumerate() {
            let value_path = format!("{path}.values[{value_position}]");
            if !valid_value(value.as_str()) {
                invalid(
                    &value_path,
                    "axis value must be a portable matrix token",
                    issues,
                );
            }
            if !values.insert(value) {
                duplicate(&value_path, value.as_str(), issues);
            }
        }
    }
}

fn valid_value(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn duplicate(path: &str, value: &str, issues: &mut Vec<ProjectIssue>) {
    issues.push(ProjectIssue::new(
        IssueCode::DuplicateId,
        path,
        format!("duplicate id {value:?}"),
    ));
}

fn invalid(path: &str, message: &str, issues: &mut Vec<ProjectIssue>) {
    issues.push(ProjectIssue::new(IssueCode::InvalidMatrix, path, message));
}
