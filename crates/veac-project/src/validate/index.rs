use std::collections::{BTreeMap, BTreeSet};

use crate::{IssueCode, ProjectIssue, ProjectManifestV1, ProjectTarget};

pub(crate) struct ManifestIndex<'a> {
    pub profiles: BTreeSet<&'a str>,
    pub locales: BTreeSet<&'a str>,
    pub targets: BTreeMap<&'a str, &'a ProjectTarget>,
}

impl<'a> ManifestIndex<'a> {
    pub fn build(manifest: &'a ProjectManifestV1, issues: &mut Vec<ProjectIssue>) -> Self {
        let mut profiles = BTreeSet::new();
        for (index, profile) in manifest.profiles.iter().enumerate() {
            insert(
                &mut profiles,
                profile.id.as_str(),
                format!("profiles[{index}].id"),
                issues,
            );
        }
        let mut locales = BTreeSet::new();
        for (index, locale) in manifest.locales.iter().enumerate() {
            insert(
                &mut locales,
                locale.id.as_str(),
                format!("locales[{index}].id"),
                issues,
            );
        }
        let mut targets = BTreeMap::new();
        for (index, target) in manifest.targets.iter().enumerate() {
            if targets.insert(target.id.as_str(), target).is_some() {
                duplicate(format!("targets[{index}].id"), target.id.as_str(), issues);
            }
        }
        Self {
            profiles,
            locales,
            targets,
        }
    }
}

fn insert<'a>(
    values: &mut BTreeSet<&'a str>,
    value: &'a str,
    path: String,
    issues: &mut Vec<ProjectIssue>,
) {
    if !values.insert(value) {
        duplicate(path, value, issues);
    }
}

fn duplicate(path: String, value: &str, issues: &mut Vec<ProjectIssue>) {
    issues.push(ProjectIssue::new(
        IssueCode::DuplicateId,
        path,
        format!("duplicate id {value:?}"),
    ));
}
