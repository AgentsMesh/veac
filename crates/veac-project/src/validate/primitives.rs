use crate::{
    ExecutionPolicy, IssueCode, ProjectIssue, ProjectManifestV1, ProjectProfile, SegmentationPolicy,
};

use super::ManifestIndex;

pub(crate) fn check_manifest_primitives(
    manifest: &ProjectManifestV1,
    index: &ManifestIndex<'_>,
    issues: &mut Vec<ProjectIssue>,
) {
    let roots = [
        (&manifest.paths.source_base, "paths.source_base"),
        (&manifest.paths.material_root, "paths.material_root"),
        (&manifest.paths.build_root, "paths.build_root"),
        (&manifest.paths.cache_root, "paths.cache_root"),
        (&manifest.paths.delivery_root, "paths.delivery_root"),
    ];
    for &(name, path) in &roots {
        check_path(name.as_str(), path, true, issues);
    }
    check_root_authorities(&roots, issues);
    check_defaults(manifest, index, issues);
    for (position, locale) in manifest.locales.iter().enumerate() {
        check_id(
            locale.id.as_str(),
            &format!("locales[{position}].id"),
            issues,
        );
        check_language_tag(&locale.language_tag, position, issues);
    }
    for (position, profile) in manifest.profiles.iter().enumerate() {
        check_id(
            profile.id.as_str(),
            &format!("profiles[{position}].id"),
            issues,
        );
        check_profile(profile, position, issues);
    }
}

fn check_root_authorities(roots: &[(&crate::ProjectPath, &str)], issues: &mut Vec<ProjectIssue>) {
    for (position, (root, path)) in roots.iter().enumerate() {
        for (other, other_path) in &roots[..position] {
            if paths_overlap(root.as_str(), other.as_str()) {
                issues.push(ProjectIssue::new(
                    IssueCode::InvalidPath,
                    *path,
                    format!("root authority overlaps {other_path}"),
                ));
            }
        }
    }
}

fn paths_overlap(left: &str, right: &str) -> bool {
    left == "."
        || right == "."
        || left == right
        || left
            .strip_prefix(right)
            .is_some_and(|rest| rest.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|rest| rest.starts_with('/'))
}

pub(crate) fn check_id(value: &str, path: &str, issues: &mut Vec<ProjectIssue>) {
    let mut bytes = value.bytes();
    let valid = value.len() <= 64
        && bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if !valid {
        issues.push(ProjectIssue::new(
            IssueCode::InvalidId,
            path,
            "id must be a 1-64 character lowercase kebab identifier",
        ));
    }
}

pub(crate) fn check_path(
    value: &str,
    path: &str,
    allow_root: bool,
    issues: &mut Vec<ProjectIssue>,
) {
    if valid_path(value, allow_root) {
        return;
    }
    issues.push(ProjectIssue::new(
        IssueCode::InvalidPath,
        path,
        "path must be canonical, project-relative, slash-separated, and traversal-free",
    ));
}

pub(crate) fn valid_path(value: &str, allow_root: bool) -> bool {
    if allow_root && value == "." {
        return true;
    }
    if value.is_empty()
        || value.starts_with('/')
        || value.contains(['\\', '\0'])
        || value.chars().any(char::is_control)
        || value.as_bytes().get(1).copied() == Some(b':')
    {
        return false;
    }
    value
        .split('/')
        .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn check_defaults(
    manifest: &ProjectManifestV1,
    index: &ManifestIndex<'_>,
    issues: &mut Vec<ProjectIssue>,
) {
    if let Some(profile) = &manifest.defaults.profile {
        if !index.profiles.contains(profile.as_str()) {
            unknown("defaults.profile", "profile", profile.as_str(), issues);
        }
    }
    if let Some(locale) = &manifest.defaults.locale {
        if !index.locales.contains(locale.as_str()) {
            unknown("defaults.locale", "locale", locale.as_str(), issues);
        }
    }
    if manifest.defaults.max_instances_per_target == 0 || manifest.defaults.max_total_instances == 0
    {
        issues.push(ProjectIssue::new(
            IssueCode::InvalidMatrix,
            "defaults",
            "matrix budgets must be positive",
        ));
    }
}

fn unknown(path: &str, kind: &str, value: &str, issues: &mut Vec<ProjectIssue>) {
    issues.push(ProjectIssue::new(
        IssueCode::UnknownReference,
        path,
        format!("unknown {kind} {value:?}"),
    ));
}

fn check_language_tag(value: &str, position: usize, issues: &mut Vec<ProjectIssue>) {
    let valid = !value.is_empty()
        && value.len() <= 63
        && value.split('-').all(|part| {
            !part.is_empty()
                && part.len() <= 8
                && part.bytes().all(|byte| byte.is_ascii_alphanumeric())
        });
    if !valid {
        issues.push(ProjectIssue::new(
            IssueCode::InvalidLocale,
            format!("locales[{position}].language_tag"),
            "language tag must use canonical BCP-47 components",
        ));
    }
}

fn check_profile(value: &ProjectProfile, position: usize, issues: &mut Vec<ProjectIssue>) {
    let execution = match value.execution {
        ExecutionPolicy::Serial {} => true,
        ExecutionPolicy::Parallel { max_tasks } => max_tasks > 0,
        ExecutionPolicy::ResourceAware {
            max_tasks,
            cpu_threads,
            memory_mib,
            ..
        } => max_tasks > 0 && cpu_threads > 0 && memory_mib > 0,
    };
    let segmentation = match value.segmentation {
        SegmentationPolicy::Whole {} => true,
        SegmentationPolicy::Fixed { duration } => duration.is_canonical() && duration.numerator > 0,
        SegmentationPolicy::Automatic { max_segments } => max_segments > 0,
    };
    let valid = execution && segmentation;
    if !valid {
        issues.push(ProjectIssue::new(
            IssueCode::InvalidProfile,
            format!("profiles[{position}].kind"),
            "profile dimensions, rates, channels, and codecs must be positive and non-empty",
        ));
    }
}
