use super::VerifiedPackageMountSet;
use crate::program::FileSystemLoader;

#[test]
fn exact_dependency_split_brain_is_rejected_independent_of_root_order() {
    let temp = tempfile::tempdir().unwrap();
    let owner = temp.path().join("owner");
    let shared = temp.path().join("shared");
    support::write_with_shared(&owner, "owner", "module {}\n");
    support::write_package(&shared, "shared", "module {}\n\n");

    for roots in [
        vec![owner.clone(), shared.clone()],
        vec![shared.clone(), owner.clone()],
    ] {
        let error = VerifiedPackageMountSet::capture(&roots).unwrap_err();
        assert_eq!(error.kind(), super::super::PackageErrorKind::Contract);
        assert!(error.message().contains("conflicting content or API"));
    }
}

#[test]
fn loader_is_bound_to_the_captured_contract_across_a_root_swap() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("package");
    let alternate = temp.path().join("alternate");
    let original = temp.path().join("original");
    support::write_package(&root, "mutable", "module {}\n");
    support::write_package(&alternate, "mutable", "module {}\n\n");
    let packages = VerifiedPackageMountSet::capture(std::slice::from_ref(&root)).unwrap();
    let project_path = temp.path().join("project/main.veac");
    std::fs::create_dir_all(project_path.parent().unwrap()).unwrap();
    std::fs::write(&project_path, "module {}\n").unwrap();
    let (project, _) = FileSystemLoader::for_entry(&project_path).unwrap();

    let result = packages.loader_with(project, || {
        std::fs::rename(&root, &original).unwrap();
        std::fs::rename(&alternate, &root).unwrap();
    });
    std::fs::rename(&root, &alternate).unwrap();
    std::fs::rename(&original, &root).unwrap();

    let error = result.unwrap_err();
    assert!(error.message().contains("digest") || error.message().contains("identity"));
    packages.revalidate().unwrap();
}

#[test]
fn mount_roots_are_canonical_ordered_and_revalidated() {
    let temp = tempfile::tempdir().unwrap();
    let alpha = temp.path().join("alpha");
    let zeta = temp.path().join("zeta");
    support::write_package(&alpha, "alpha", "module {}\n");
    support::write_package(&zeta, "zeta", "module {}\n");
    let packages = VerifiedPackageMountSet::capture(&[zeta, alpha]).unwrap();
    let names = packages
        .discoveries()
        .map(|item| item.root.package.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["alpha", "zeta"]);
    packages.revalidate().unwrap();
}

#[test]
fn duplicate_identity_and_overlapping_roots_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    support::write_package(&first, "same", "module {}\n");
    support::write_package(&second, "same", "module {}\n");
    let error = VerifiedPackageMountSet::capture(&[first, second]).unwrap_err();
    assert!(error.message().contains("duplicate exact identity"));

    let outer = temp.path().join("outer");
    let inner = outer.join("nested");
    support::write_package(&outer, "outer", "module {}\n");
    support::write_package(&inner, "inner", "module {}\n");
    let error = VerifiedPackageMountSet::capture(&[outer, inner]).unwrap_err();
    assert!(error.message().contains("must not overlap"));
}

#[test]
fn equal_locked_dependency_contracts_can_be_shared_across_mounts() {
    let temp = tempfile::tempdir().unwrap();
    let alpha = temp.path().join("alpha");
    let beta = temp.path().join("beta");
    support::write_with_shared(&alpha, "alpha", "module {}\n");
    support::write_with_shared(&beta, "beta", "module {}\n");
    VerifiedPackageMountSet::capture(&[alpha, beta]).unwrap();
}

#[test]
fn successful_loader_uses_the_captured_mount_and_drift_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("package");
    support::write_package(&root, "stable", "module {}\n");
    let packages = VerifiedPackageMountSet::capture(std::slice::from_ref(&root)).unwrap();
    let project_path = temp.path().join("project/main.veac");
    std::fs::create_dir_all(project_path.parent().unwrap()).unwrap();
    std::fs::write(&project_path, "module {}\n").unwrap();
    let (project, _) = FileSystemLoader::for_entry(&project_path).unwrap();
    let loader = packages.loader(project).unwrap();
    let entry = loader
        .load_package_entry(&support::identity("stable"))
        .unwrap();
    assert!(entry.id.contains("stable@1.0.0/main.veac"));

    support::write_package(&root, "stable", "module {}\n\n");
    let error = packages.revalidate().unwrap_err();
    assert!(error.message().contains("changed after mount-set capture"));
}

#[path = "tests/support.rs"]
mod support;
