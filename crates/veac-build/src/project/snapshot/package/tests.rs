use veac_artifact::ContentDigest;
use veac_project::{TargetId, TargetInstanceId};

use super::ProjectPackageSet;
use crate::{
    ProjectAction, ProjectComputation, ProjectFileSnapshot, ProjectPackageMountRevision,
    ProjectSourceGraphRevision,
};

#[test]
fn action_json_contains_a_portable_closed_package_contract() {
    let dependency = support::revision("util", 20, Vec::new());
    let root = support::revision("components", 10, vec![dependency.package.clone()]);
    let action = action(vec![ProjectPackageMountRevision {
        root,
        dependencies: vec![dependency],
    }]);

    let value = serde_json::to_value(action).unwrap();
    let mount = &value["computation"]["package_mounts"][0];
    assert_eq!(mount["root"]["package"]["name"], "components");
    assert_eq!(mount["root"]["package"]["version"], "1.0.0");
    assert_eq!(mount["root"]["entry_module"], "main.veac");
    assert!(mount["root"]["entry_sha256"].as_str().is_some());
    assert!(mount["root"]["content_sha256"].as_str().is_some());
    assert!(mount["root"]["api_sha256"].as_str().is_some());
    assert_eq!(mount["root"]["dependencies"][0]["name"], "util");
    assert_eq!(mount["dependencies"][0]["package"]["name"], "util");
    let json = serde_json::to_string(&value).unwrap();
    assert!(!json.contains("/host/packages"));
    assert!(!json.contains("host_root"));
}

#[test]
fn explicit_root_order_does_not_change_the_revision() {
    let temp = tempfile::tempdir().unwrap();
    let alpha = temp.path().join("alpha");
    let zeta = temp.path().join("zeta");
    support::write_package(&alpha, "alpha", "module {}\n");
    support::write_package(&zeta, "zeta", "module {}\n");

    let forward = ProjectPackageSet::capture(&[zeta.clone(), alpha.clone()]).unwrap();
    let reverse = ProjectPackageSet::capture(&[alpha, zeta]).unwrap();
    assert_eq!(forward.revision(), reverse.revision());
    assert_eq!(forward.revision()[0].root.package.name.as_str(), "alpha");
    assert_eq!(forward.host_roots().count(), 2);
    assert!(forward.require_revision(forward.revision()).is_ok());
    assert!(forward.require_revision(&[]).is_err());
}

#[test]
fn duplicate_exact_root_identity_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    support::write_package(&first, "same", "module {}\n");
    support::write_package(&second, "same", "module {}\n");

    let error = ProjectPackageSet::capture(&[first, second]).unwrap_err();
    assert!(error.message().contains("duplicate exact identity"));
}

#[test]
fn overlapping_package_roots_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let outer = temp.path().join("outer");
    let inner = outer.join("nested");
    support::write_package(&outer, "outer", "module {}\n");
    support::write_package(&inner, "inner", "module {}\n");

    let error = ProjectPackageSet::capture(&[outer, inner]).unwrap_err();
    assert!(error.message().contains("must not overlap"));
}

#[test]
fn valid_content_and_api_drift_invalidates_a_captured_set() {
    for changed in [
        "module {}\n\n",
        "module { export fn value() -> int { 1 } }\n",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("package");
        support::write_package(&root, "mutable", "module {}\n");
        let packages = ProjectPackageSet::capture(std::slice::from_ref(&root)).unwrap();

        support::write_package(&root, "mutable", changed);
        let error = packages.revalidate().unwrap_err();
        assert!(error.message().contains("changed after mount-set capture"));
    }
}

fn action(package_mounts: Vec<ProjectPackageMountRevision>) -> ProjectAction {
    ProjectAction::Evidence {
        computation: ProjectComputation {
            instance: TargetInstanceId::from("instance"),
            target: TargetId::from("target"),
            profile: None,
            locale: None,
            matrix: Default::default(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            bound_sources: Vec::new(),
            package_mounts,
        },
        contract: ProjectFileSnapshot {
            path: "evidence.veac".to_owned(),
            content: ContentDigest::sha256(b"evidence"),
            size_bytes: 8,
        },
        source_graph: ProjectSourceGraphRevision {
            root_module: "evidence.veac".to_owned(),
            authored_source_graph_sha256: "1".repeat(64),
            complete_source_graph_sha256: "2".repeat(64),
            authored_module_count: 1,
            authored_modules: vec!["evidence.veac".to_owned()],
        },
    }
}

#[path = "support.rs"]
mod support;
