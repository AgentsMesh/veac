use crate::{
    InstanceSelector, IssueCode, ProjectIssue, ProjectManifestV1, ProjectTarget, TargetRef,
};

use super::{check_id, ManifestIndex};

pub(crate) fn check_target_ref(
    manifest: &ProjectManifestV1,
    index: &ManifestIndex<'_>,
    reference: &TargetRef,
    path: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    check_id(reference.target.as_str(), &format!("{path}.target"), issues);
    let Some(target) = index.targets.get(reference.target.as_str()) else {
        unknown(
            &format!("{path}.target"),
            "target",
            reference.target.as_str(),
            issues,
        );
        return;
    };
    if let Some(profile) = &reference.profile {
        let effective = if target.profiles.is_empty() {
            manifest
                .defaults
                .profile
                .as_ref()
                .is_some_and(|value| value == profile)
        } else {
            target.profiles.contains(profile)
        };
        if !index.profiles.contains(profile.as_str()) || !effective {
            unknown(
                &format!("{path}.profile"),
                "target profile",
                profile.as_str(),
                issues,
            );
        }
    }
    if let InstanceSelector::Exact { locale, axes } = &reference.selector {
        if let Some(locale) = locale {
            if !target.localized || !index.locales.contains(locale.as_str()) {
                unknown(
                    &format!("{path}.selector.locale"),
                    "target locale",
                    locale.as_str(),
                    issues,
                );
            }
        }
        for (axis, value) in axes {
            let valid = target.axes.iter().any(|item| {
                item.id == *axis && item.values.iter().any(|candidate| candidate == value)
            });
            if !valid {
                unknown(
                    &format!("{path}.selector.axes.{}", axis.as_str()),
                    "target matrix value",
                    value.as_str(),
                    issues,
                );
            }
        }
    }
}

pub(crate) fn check_output_ref(
    target: &ProjectTarget,
    output: &crate::OutputId,
    path: &str,
    issues: &mut Vec<ProjectIssue>,
) {
    if !target
        .outputs
        .iter()
        .any(|candidate| candidate.id() == output)
    {
        unknown(path, "target output", output.as_str(), issues);
    }
}

fn unknown(path: &str, kind: &str, value: &str, issues: &mut Vec<ProjectIssue>) {
    issues.push(ProjectIssue::new(
        IssueCode::UnknownReference,
        path,
        format!("unknown {kind} {value:?}"),
    ));
}
