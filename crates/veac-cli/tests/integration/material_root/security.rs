use super::super::support::veac;
use super::support::DetachedFixture;

#[test]
fn material_root_conflicts_with_bindings_for_project_consumers() {
    let cases: &[(&str, &[&str])] = &[
        ("plan", &[]),
        ("manifest", &[]),
        ("bundle", &["--destination", "package"]),
        ("render", &[]),
    ];
    for (name, tail) in cases {
        let output = veac()
            .arg(name)
            .arg("project.json")
            .args(["--material-root", "materials"])
            .args(["--bindings", "bindings.json"])
            .args(*tail)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{name}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("--material-root"), "{name}: {stderr}");
        assert!(stderr.contains("--bindings"), "{name}: {stderr}");
    }
}

#[test]
fn probe_material_root_requires_canonical_material_mode() {
    let output = veac()
        .args(["probe", "clip.mp4", "--material-root", "materials"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--material"), "{stderr}");
}

#[test]
fn detached_ir_without_material_root_keeps_the_project_directory_default() {
    let fixture = DetachedFixture::new();
    fixture.build_success();
    veac()
        .arg("plan")
        .arg(&fixture.project)
        .assert()
        .failure()
        .stderr(predicates::str::contains("PATH_UNAVAILABLE"));
}

#[test]
fn missing_and_non_directory_material_roots_fail_closed() {
    let fixture = DetachedFixture::new();
    fixture.build_success();
    let missing = fixture.workspace().join("missing");
    fixture
        .command_at_root("plan", &missing)
        .assert()
        .failure()
        .stderr(predicates::str::contains("PATH_UNAVAILABLE"));
    fixture
        .command_at_root("plan", &fixture.source)
        .assert()
        .failure()
        .stderr(predicates::str::contains("NOT_A_DIRECTORY"));
}

#[cfg(unix)]
#[test]
fn material_root_alias_is_allowed_but_internal_escape_is_rejected() {
    use std::os::unix::fs::symlink;

    let fixture = DetachedFixture::new();
    fixture.build_success();
    let alias = fixture.workspace().join("source-alias");
    symlink(&fixture.source_root, &alias).unwrap();
    fixture.command_at_root("plan", &alias).assert().success();

    let confined = fixture.workspace().join("confined");
    std::fs::create_dir(&confined).unwrap();
    symlink(&fixture.media, confined.join("clip.mp4")).unwrap();
    fixture
        .command_at_root("plan", &confined)
        .assert()
        .failure()
        .stderr(predicates::str::contains("MATERIAL_OUTSIDE_ROOT"));
}

#[test]
fn package_rejects_directories_that_contain_project_inputs() {
    let fixture = DetachedFixture::new();
    fixture.build_success();
    let media_before = std::fs::read(&fixture.media).unwrap();
    let project_before = std::fs::read(&fixture.project).unwrap();
    for destination in [&fixture.source_root, &fixture.build_root] {
        fixture
            .command("bundle")
            .arg("--destination")
            .arg(destination)
            .assert()
            .failure()
            .stderr(predicates::str::contains("OUTPUT_CONTAINS_INPUT"));
    }
    assert!(!fixture.source_root.join("package.json").exists());
    assert!(!fixture.build_root.join("package.json").exists());
    assert_eq!(std::fs::read(&fixture.media).unwrap(), media_before);
    assert_eq!(std::fs::read(&fixture.project).unwrap(), project_before);
}
