use crate::{IssueCode, ProjectIssue, ProjectManifestV1, ProjectTarget};

pub(crate) fn check_budgets(manifest: &ProjectManifestV1, issues: &mut Vec<ProjectIssue>) {
    let mut total = 0_u64;
    for (position, target) in manifest.targets.iter().enumerate() {
        let count = instance_count(manifest, target);
        total = total.saturating_add(count);
        if count > u64::from(manifest.defaults.max_instances_per_target) {
            issues.push(ProjectIssue::new(
                IssueCode::MatrixBudgetExceeded,
                format!("targets[{position}]"),
                format!(
                    "matrix expands to {count} instances, exceeding per-target budget {}",
                    manifest.defaults.max_instances_per_target
                ),
            ));
        }
    }
    if total > u64::from(manifest.defaults.max_total_instances) {
        issues.push(ProjectIssue::new(
            IssueCode::MatrixBudgetExceeded,
            "targets",
            format!(
                "matrix expands to {total} total instances, exceeding total budget {}",
                manifest.defaults.max_total_instances
            ),
        ));
    }
}

pub(crate) fn instance_count(manifest: &ProjectManifestV1, target: &ProjectTarget) -> u64 {
    let profiles = if target.profiles.is_empty() {
        1
    } else {
        target.profiles.len()
    };
    let locales = if target.localized {
        manifest.locales.len()
    } else {
        1
    };
    target.axes.iter().fold(
        (profiles as u64).saturating_mul(locales as u64),
        |count, axis| count.saturating_mul(axis.values.len() as u64),
    )
}
