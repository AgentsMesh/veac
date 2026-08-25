use super::*;
use crate::package::validation;

#[path = "tests/api_projection.rs"]
mod api_projection_tests;
#[path = "tests/api.rs"]
mod api_tests;
#[path = "tests/contract_edges.rs"]
mod contract_edge_tests;
#[path = "tests/contracts.rs"]
mod contract_tests;
#[path = "tests/graph.rs"]
mod graph_tests;
#[path = "tests/io.rs"]
mod io_tests;
#[path = "tests/source_loader.rs"]
mod source_loader_tests;

fn name(value: &str) -> PackageName {
    PackageName::new(value).unwrap()
}

fn version(value: &str) -> ExactVersion {
    ExactVersion::new(value).unwrap()
}

fn digest(value: char) -> Sha256Digest {
    Sha256Digest::parse(&value.to_string().repeat(64)).unwrap()
}

fn identity(package: &str, exact: &str) -> PackageIdentity {
    PackageIdentity {
        name: name(package),
        version: version(exact),
    }
}

fn dependency(package: &str, exact: &str) -> PackageDependency {
    PackageDependency {
        name: name(package),
        version: version(exact),
    }
}

fn locked(package: &str, exact: &str, dependencies: Vec<PackageDependency>) -> LockedPackage {
    let files = vec![LockedFile {
        path: "lib.veac".to_owned(),
        sha256: digest('a'),
    }];
    LockedPackage {
        package: identity(package, exact),
        path: format!("vendor/{package}"),
        entry: "lib.veac".to_owned(),
        dependencies,
        content_sha256: package_content_digest(&files).unwrap(),
        files,
        api_sha256: digest('b'),
    }
}

fn root_files() -> Vec<LockedFile> {
    vec![LockedFile {
        path: "main.veac".to_owned(),
        sha256: digest('c'),
    }]
}

fn package_lock(root: PackageIdentity, packages: Vec<LockedPackage>) -> PackageLockV1 {
    PackageLockV1::new(root, root_files(), packages).unwrap()
}
