mod action;
mod binding;
mod derivation;
mod index;
mod matrix;
mod primitives;
mod reference;
mod target;
mod target_matrix;
mod template;

use crate::{IssueCode, ProjectIssue, ProjectIssues, ProjectManifestV1};

pub(crate) use index::ManifestIndex;
pub(crate) use primitives::{check_id, check_path};
pub(crate) use reference::check_target_ref;
pub(crate) use template::{expand_template, validate_template};

pub fn validate_manifest(manifest: &ProjectManifestV1) -> Result<(), ProjectIssues> {
    let mut issues = Vec::new();
    check_header(manifest, &mut issues);
    let index = ManifestIndex::build(manifest, &mut issues);
    primitives::check_manifest_primitives(manifest, &index, &mut issues);
    for (position, target) in manifest.targets.iter().enumerate() {
        target::check_target(manifest, &index, target, position, &mut issues);
    }
    matrix::check_budgets(manifest, &mut issues);
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ProjectIssues::sorted(issues))
    }
}

fn check_header(manifest: &ProjectManifestV1, issues: &mut Vec<ProjectIssue>) {
    if manifest.schema != crate::PROJECT_SCHEMA {
        issues.push(ProjectIssue::new(
            IssueCode::InvalidSchema,
            "schema",
            format!("schema must be {:?}", crate::PROJECT_SCHEMA),
        ));
    }
    if manifest.version != crate::PROJECT_MANIFEST_VERSION {
        issues.push(ProjectIssue::new(
            IssueCode::InvalidVersion,
            "version",
            format!("version must be {}", crate::PROJECT_MANIFEST_VERSION),
        ));
    }
    check_id(manifest.id.as_str(), "id", issues);
}
