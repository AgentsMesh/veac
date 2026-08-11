mod support;

use support::{assert_code, manifest};
use veac_project::*;

#[test]
fn reference_manifest_is_valid() {
    validate_manifest(&manifest()).unwrap();
}

#[test]
fn schema_version_and_ids_are_strict() {
    let mut value = manifest();
    value.schema = "veac.project.other".to_owned();
    value.version = 2;
    value.id = ProjectId::from("Not_valid");
    value.locales[0].id = LocaleId::from("");
    value.profiles[0].id = ProfileId::from("a_very_bad_id");
    value.targets[0].id = TargetId::new(format!("a{}", "x".repeat(64)));
    let error = validate_manifest(&value).unwrap_err();
    let codes = support::codes(&error);
    assert!(codes.contains(&IssueCode::InvalidSchema));
    assert!(codes.contains(&IssueCode::InvalidVersion));
    assert!(codes.contains(&IssueCode::InvalidId));
    assert_eq!(
        error.to_string(),
        format!("project contract has {} issue(s)", error.issues().len())
    );
}

#[test]
fn all_project_paths_are_portable_and_relative() {
    for invalid in [
        "",
        "/absolute",
        "../escape",
        "nested/../escape",
        "nested//file",
        "nested/./file",
        "C:/drive",
        "folder\\file",
        "line\nbreak",
    ] {
        let mut value = manifest();
        value.paths.material_root = ProjectPath::new(invalid);
        assert_code(&value, IssueCode::InvalidPath);
    }
    let mut root_entry = manifest();
    root_entry.targets[0].entry = ProjectTargetEntry::Veac {
        source: ProjectPath::new("."),
    };
    assert_code(&root_entry, IssueCode::InvalidPath);
}

#[test]
fn project_root_authorities_must_not_overlap() {
    for overlapping in [".", "sources", "sources/generated", "materials/raw"] {
        let mut value = manifest();
        value.paths.build_root = ProjectPath::new(overlapping);
        assert_code(&value, IssueCode::InvalidPath);
    }

    let mut value = manifest();
    value.paths.cache_root = value.paths.delivery_root.clone();
    assert_code(&value, IssueCode::InvalidPath);
}

#[test]
fn duplicate_top_level_ids_and_unknown_defaults_fail() {
    let mut value = manifest();
    value.locales.push(value.locales[0].clone());
    value.profiles.push(value.profiles[0].clone());
    value.targets.push(value.targets[0].clone());
    value.defaults.profile = Some(ProfileId::from("missing"));
    value.defaults.locale = Some(LocaleId::from("missing"));
    value.defaults.max_instances_per_target = 0;
    value.defaults.max_total_instances = 0;
    let error = validate_manifest(&value).unwrap_err();
    let codes = support::codes(&error);
    assert!(codes.contains(&IssueCode::DuplicateId));
    assert!(codes.contains(&IssueCode::UnknownReference));
    assert!(codes.contains(&IssueCode::InvalidMatrix));
}

#[test]
fn locale_tags_are_checked() {
    for invalid in ["", "zh--CN", "toolongcomponent", "zh_中文"] {
        let mut value = manifest();
        value.locales[0].language_tag = invalid.to_owned();
        assert_code(&value, IssueCode::InvalidLocale);
    }
}

#[test]
fn execution_proxy_and_segmentation_profiles_are_closed_and_validated() {
    let mut value = manifest();
    value.profiles[0].execution = ExecutionPolicy::ResourceAware {
        max_tasks: 0,
        cpu_threads: 0,
        memory_mib: 0,
        gpu_slots: 0,
    };
    assert_code(&value, IssueCode::InvalidProfile);

    value.profiles[0].execution = ExecutionPolicy::Parallel { max_tasks: 1 };
    value.profiles[0].segmentation = SegmentationPolicy::Automatic { max_segments: 0 };
    assert_code(&value, IssueCode::InvalidProfile);

    value.profiles[0].execution = ExecutionPolicy::Serial {};
    value.profiles[0].proxy = ProxyPolicy::Require {};
    value.profiles[0].segmentation = SegmentationPolicy::Fixed {
        duration: ProjectRational::new(5, 1),
    };
    validate_manifest(&value).unwrap();
}

#[test]
fn matrix_shape_and_budgets_are_bounded() {
    let mut value = manifest();
    value.defaults.max_instances_per_target = 1;
    value.defaults.max_total_instances = 2;
    assert_code(&value, IssueCode::MatrixBudgetExceeded);

    let mut invalid = manifest();
    invalid.targets[0].axes[0].values.clear();
    invalid.targets[0].axes.push(MatrixAxis {
        id: AxisId::from("theme"),
        values: vec![AxisValue::from("bad/value"), AxisValue::from("bad/value")],
    });
    invalid.targets[0].profiles.push(ProfileId::from("preview"));
    let error = validate_manifest(&invalid).unwrap_err();
    let codes = support::codes(&error);
    assert!(codes.contains(&IssueCode::InvalidMatrix));
    assert!(codes.contains(&IssueCode::DuplicateId));

    invalid.targets[0].axes = (0..17)
        .map(|index| MatrixAxis {
            id: AxisId::new(format!("axis-{index}")),
            values: vec![AxisValue::from("one")],
        })
        .collect();
    assert_code(&invalid, IssueCode::InvalidMatrix);
}

#[test]
fn localized_target_requires_locales() {
    let mut value = manifest();
    value.locales.clear();
    value.defaults.locale = None;
    assert_code(&value, IssueCode::InvalidMatrix);
}
