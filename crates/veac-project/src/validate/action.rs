use crate::{ProjectIssue, ProjectTarget, ProjectTargetEntry};

use super::check_path;

pub(crate) fn check_entry(target: &ProjectTarget, base: &str, issues: &mut Vec<ProjectIssue>) {
    match &target.entry {
        ProjectTargetEntry::Veac { source } => {
            check_path(
                source.as_str(),
                &format!("{base}.entry.source"),
                false,
                issues,
            );
        }
        ProjectTargetEntry::Evidence { contract } => {
            check_path(
                contract.as_str(),
                &format!("{base}.entry.contract"),
                false,
                issues,
            );
        }
        ProjectTargetEntry::MediaDerivation { operation } => {
            super::derivation::check(target, operation, base, issues);
        }
    }
}
