use std::collections::BTreeSet;

use crate::{IssueCode, ProjectIssue, ProjectManifestV1, ProjectTarget};

use super::{
    check_id, check_target_ref, reference::check_output_ref, validate_template, ManifestIndex,
};

pub(crate) fn check_target(
    manifest: &ProjectManifestV1,
    index: &ManifestIndex<'_>,
    target: &ProjectTarget,
    position: usize,
    issues: &mut Vec<ProjectIssue>,
) {
    let base = format!("targets[{position}]");
    check_id(target.id.as_str(), &format!("{base}.id"), issues);
    super::action::check_entry(target, &base, issues);
    if target.localized && manifest.locales.is_empty() {
        issues.push(ProjectIssue::new(
            IssueCode::InvalidMatrix,
            format!("{base}.localized"),
            "localized target requires at least one locale",
        ));
    }
    super::target_matrix::check_profiles(index, target, &base, issues);
    super::target_matrix::check_axes(target, &base, issues);
    super::binding::check_inputs(manifest, index, target, &base, issues);
    check_needs(manifest, index, target, &base, issues);
    check_outputs(target, &base, issues);
    check_deliveries(manifest, target, &base, issues);
}

fn check_needs(
    manifest: &ProjectManifestV1,
    index: &ManifestIndex<'_>,
    target: &ProjectTarget,
    base: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    for (position, need) in target.needs.iter().enumerate() {
        check_target_ref(
            manifest,
            index,
            need,
            &format!("{base}.needs[{position}]"),
            issues,
        );
    }
}

fn check_outputs(target: &ProjectTarget, base: &str, issues: &mut Vec<ProjectIssue>) {
    let mut outputs = BTreeSet::new();
    for (position, output) in target.outputs.iter().enumerate() {
        check_id(
            output.id().as_str(),
            &format!("{base}.outputs[{position}].id"),
            issues,
        );
        if !outputs.insert(output.id()) {
            duplicate(
                &format!("{base}.outputs[{position}].id"),
                output.id().as_str(),
                issues,
            );
        }
    }
}

fn check_deliveries(
    manifest: &ProjectManifestV1,
    target: &ProjectTarget,
    base: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    let mut deliveries = BTreeSet::new();
    for (position, delivery) in target.deliveries.iter().enumerate() {
        let path = format!("{base}.deliveries[{position}]");
        check_id(delivery.id().as_str(), &format!("{path}.id"), issues);
        if !deliveries.insert(delivery.id()) {
            duplicate(&format!("{path}.id"), delivery.id().as_str(), issues);
        }
        check_output_ref(target, delivery.output(), &format!("{path}.output"), issues);
        validate_template(
            delivery.destination().as_str(),
            target,
            manifest.defaults.profile.is_some(),
            &format!("{path}.destination"),
            issues,
        );
    }
}

fn duplicate(path: &str, value: &str, issues: &mut Vec<ProjectIssue>) {
    issues.push(ProjectIssue::new(
        IssueCode::DuplicateId,
        path,
        format!("duplicate id {value:?}"),
    ));
}
