use std::collections::BTreeSet;

use crate::{IssueCode, ProjectInputSource, ProjectIssue, ProjectManifestV1, ProjectTarget};

use super::{check_id, check_path, check_target_ref, reference::check_output_ref, ManifestIndex};

pub(crate) fn check_inputs(
    manifest: &ProjectManifestV1,
    index: &ManifestIndex<'_>,
    target: &ProjectTarget,
    base: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    let mut bindings = BTreeSet::new();
    for (position, input) in target.inputs.iter().enumerate() {
        let path = format!("{base}.inputs[{position}]");
        check_id(input.id.as_str(), &format!("{path}.id"), issues);
        if !bindings.insert(&input.id) {
            issues.push(ProjectIssue::new(
                IssueCode::BindingConflict,
                format!("{path}.id"),
                format!(
                    "input binding {:?} is declared more than once",
                    input.id.as_str()
                ),
            ));
        }
        check_source(manifest, index, target, &input.source, &path, issues);
    }
}

fn check_source(
    manifest: &ProjectManifestV1,
    index: &ManifestIndex<'_>,
    owner: &ProjectTarget,
    source: &ProjectInputSource,
    path: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    match source {
        ProjectInputSource::Literal { value } => check_literal(value, path, issues),
        ProjectInputSource::ProfileBinding {} => {
            if owner.profiles.is_empty() && manifest.defaults.profile.is_none() {
                binding_error(path, "profile binding requires a target profile", issues);
            }
        }
        ProjectInputSource::LocaleBinding {} => {
            if !owner.localized {
                binding_error(path, "locale binding requires a localized target", issues);
            }
        }
        ProjectInputSource::MatrixBinding { axis } => {
            if !owner.axes.iter().any(|item| item.id == *axis) {
                binding_error(
                    path,
                    "matrix binding references an unknown target axis",
                    issues,
                );
            }
        }
        ProjectInputSource::ProjectMaterial { path: value } => {
            check_path(
                value.as_str(),
                &format!("{path}.source.path"),
                false,
                issues,
            );
        }
        ProjectInputSource::AssetFact { path: value, fact } => {
            check_path(
                value.as_str(),
                &format!("{path}.source.path"),
                false,
                issues,
            );
            check_id(fact.as_str(), &format!("{path}.source.fact"), issues);
        }
        ProjectInputSource::Artifact { target, output } => {
            check_target_ref(
                manifest,
                index,
                target,
                &format!("{path}.source.target"),
                issues,
            );
            if let Some(referenced) = index.targets.get(target.target.as_str()) {
                check_output_ref(referenced, output, &format!("{path}.source.output"), issues);
            }
        }
        ProjectInputSource::AnalysisFact {
            target,
            output,
            fact,
        } => {
            check_id(fact.as_str(), &format!("{path}.source.fact"), issues);
            check_target_ref(
                manifest,
                index,
                target,
                &format!("{path}.source.target"),
                issues,
            );
            if let Some(referenced) = index.targets.get(target.target.as_str()) {
                check_output_ref(referenced, output, &format!("{path}.source.output"), issues);
            }
        }
    }
}

fn check_literal(value: &crate::ProjectLiteral, path: &str, issues: &mut Vec<ProjectIssue>) {
    use crate::ProjectLiteral;
    let valid = match value {
        ProjectLiteral::Scalar { value } => value.is_canonical(),
        ProjectLiteral::Duration { value } => value.is_canonical() && value.numerator >= 0,
        ProjectLiteral::Text { value } => value.len() <= 65_536,
        ProjectLiteral::Identifier { value } => {
            !value.is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
        }
        ProjectLiteral::Bool { .. } | ProjectLiteral::Integer { .. } => true,
    };
    if !valid {
        issues.push(ProjectIssue::new(
            IssueCode::InvalidLiteral,
            format!("{path}.source.value"),
            "literal is outside the canonical project value contract",
        ));
    }
}

fn binding_error(path: &str, message: &str, issues: &mut Vec<ProjectIssue>) {
    issues.push(ProjectIssue::new(
        IssueCode::BindingConflict,
        format!("{path}.source"),
        message,
    ));
}
