#[path = "package_tests/io_tests.rs"]
mod io_tests;
#[path = "package_tests/package_contract_edge_tests.rs"]
mod package_contract_edge_tests;
#[path = "package_tests/package_error_tests.rs"]
mod package_error_tests;
#[path = "package_tests/path_safety_tests.rs"]
mod path_safety_tests;
#[path = "package_tests/relink_discovery_tests.rs"]
mod relink_discovery_tests;

use std::fs;

use crate::{test_support::*, *};

#[test]
fn package_copies_verified_inputs_and_restores_bindings() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source.mp4");
    fs::write(&source, b"media").unwrap();
    let plan = plan(b"media");
    let project = project(b"media");
    let bindings = original_bindings(&plan, &source);
    let destination = root.path().join("package");
    let manifest = package_plan(&project, &plan, &bindings, &destination).unwrap();
    let again = package_plan(&project, &plan, &bindings, &destination).unwrap();
    assert_eq!(manifest, again);
    assert_eq!(manifest.entries.len(), 1);
    assert_eq!(
        fs::read(destination.join("package.json")).unwrap(),
        canonical_package_bytes(&manifest).unwrap()
    );
    let portable = package_binding_manifest(&destination, &manifest).unwrap();
    let restored = resolve_binding_manifest(&plan, &portable).unwrap();
    assert_eq!(restored.inputs().len(), 1);
    assert!(restored.outputs().is_empty());
    assert_eq!(portable.plan_hash, manifest.plan_hash);
    assert!(portable.inputs[0].path.is_absolute());
    assert_eq!(
        packaged_project(&destination, &manifest).unwrap(),
        destination.join("project.veac.json")
    );
}

#[test]
fn relink_resolution_reports_exact_missing_and_ambiguous_matches() {
    let plan = plan(b"media");
    let identity = plan.inputs[0].observed_identity.clone();
    let unresolved = resolve_relinks(&plan, &[]).unwrap();
    assert_eq!(unresolved.unresolved, vec![plan.inputs[0].id.clone()]);
    let resolved = resolve_relinks(
        &plan,
        &[RelinkCandidate {
            path: "/media/a.mp4".into(),
            identity: identity.clone(),
        }],
    )
    .unwrap();
    assert_eq!(
        resolved
            .bindings
            .input(&plan.inputs[0].id)
            .unwrap()
            .resource()
            .unwrap()
            .path(),
        std::path::Path::new("/media/a.mp4")
    );
    let ambiguous = resolve_relinks(
        &plan,
        &[
            RelinkCandidate {
                path: "/media/b.mp4".into(),
                identity: identity.clone(),
            },
            RelinkCandidate {
                path: "/media/a.mp4".into(),
                identity: identity.clone(),
            },
            RelinkCandidate {
                path: "/media/a.mp4".into(),
                identity,
            },
        ],
    )
    .unwrap();
    assert_eq!(ambiguous.ambiguous.len(), 1);
    assert_eq!(ambiguous.ambiguous[0].candidates.len(), 2);
    assert!(ambiguous.bindings.inputs().is_empty());
}
