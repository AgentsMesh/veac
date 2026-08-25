use veac_lang::package::{
    canonical_package_manifest_json, discover_package, parse_package_manifest_json, sha256_bytes,
    ExactVersion, PackageIdentity, PackageName, PackageSourceLoader,
};
use veac_lang::program::{CompilerDatabase, SourceLoader};

fn identity(name: &str, version: &str) -> PackageIdentity {
    PackageIdentity {
        name: PackageName::new(name).unwrap(),
        version: ExactVersion::new(version).unwrap(),
    }
}

#[test]
fn discovery_reads_a_locked_root_and_exact_dependency_without_network_or_hooks() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    let found = discover_package(temp.path()).unwrap();
    assert_eq!(found.dependencies.len(), 1);
    assert_eq!(found.dependencies[0].package.name.as_str(), "util");
    assert!(found.root.entry.ends_with("main.veac"));
}

#[test]
fn discovery_rejects_tampering_and_root_escape() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    std::fs::write(temp.path().join("vendor/util/lib.veac"), "changed").unwrap();
    let error = discover_package(temp.path()).unwrap_err();
    assert!(error.message().contains("digest mismatch"));

    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    let manifest = std::fs::read_to_string(temp.path().join("veac.package.json")).unwrap();
    std::fs::write(
        temp.path().join("veac.package.json"),
        manifest.replace("main.veac", "../outside.veac"),
    )
    .unwrap();
    assert!(discover_package(temp.path())
        .unwrap_err()
        .message()
        .contains("root-confined"));
}

#[test]
fn discovery_rejects_root_identity_entry_and_api_drift() {
    for mutation in ["root", "entry", "api"] {
        let temp = tempfile::tempdir().unwrap();
        helpers::write_contract(temp.path());
        match mutation {
            "root" => helpers::rewrite_lock_root(temp.path(), identity("other", "1.0.0")),
            "entry" => {
                std::fs::write(temp.path().join("main.veac"), b"changed\n").unwrap();
            }
            "api" => {
                let path = temp.path().join("veac.package.json");
                let text = std::fs::read_to_string(&path).unwrap();
                let mut manifest = parse_package_manifest_json(&text).unwrap();
                manifest.api_sha256 = sha256_bytes(b"wrong API");
                std::fs::write(path, canonical_package_manifest_json(&manifest).unwrap()).unwrap();
            }
            _ => unreachable!(),
        }
        assert!(
            discover_package(temp.path()).is_err(),
            "accepted {mutation} drift"
        );
    }
}

#[test]
fn discovery_rejects_missing_contracts_and_non_utf8_sources() {
    let missing = tempfile::tempdir().unwrap();
    assert!(discover_package(missing.path()).is_err());

    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    std::fs::write(temp.path().join("vendor/util/lib.veac"), [0xff]).unwrap();
    helpers::rewrite_dependency_digest(temp.path(), &[0xff]);
    let error = PackageSourceLoader::for_root(temp.path()).unwrap_err();
    assert!(error.message().contains("not UTF-8"));
}

#[cfg(unix)]
#[test]
fn discovery_rejects_symlinked_package_content() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    std::fs::remove_file(temp.path().join("vendor/util/lib.veac")).unwrap();
    std::os::unix::fs::symlink(
        temp.path().join("main.veac"),
        temp.path().join("vendor/util/lib.veac"),
    )
    .unwrap();
    assert!(discover_package(temp.path()).is_err());
}

#[test]
fn package_source_loader_resolves_only_locked_source_ids() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    std::fs::write(temp.path().join("vendor/util/hidden.veac"), "module {}\n").unwrap();
    let (loader, entry) = PackageSourceLoader::for_root(temp.path()).unwrap();
    assert_eq!(entry.id, "packages/root@1.0.0/main.veac");
    assert_eq!(loader.entry_id(), "packages/root@1.0.0/main.veac");
    assert_eq!(loader.root(), temp.path().canonicalize().unwrap());
    let loaded = loader
        .load(
            "packages/root@1.0.0/main.veac",
            "package:util@2.0.0/lib.veac",
        )
        .unwrap();
    assert_eq!(loaded.id, "packages/util@2.0.0/lib.veac");
    assert!(loader
        .load("packages/util@2.0.0/lib.veac", "./hidden.veac")
        .is_err());
    assert!(loader
        .load("packages/root@1.0.0/main.veac", "../outside.veac")
        .is_err());
    assert_eq!(
        loader
            .load_package(&identity("root", "1.0.0"), "./main.veac")
            .unwrap()
            .id,
        "packages/root@1.0.0/main.veac"
    );
    assert!(loader.load("../bad.veac", "./missing.veac").is_err());
}

#[test]
fn package_source_loader_rechecks_locked_digest_on_every_read() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    let (loader, _) = PackageSourceLoader::for_root(temp.path()).unwrap();
    let package = identity("util", "2.0.0");
    assert_eq!(
        loader.load_package(&package, "lib.veac").unwrap().id,
        "packages/util@2.0.0/lib.veac"
    );
    std::fs::write(temp.path().join("vendor/util/lib.veac"), "changed").unwrap();
    assert!(loader
        .load_package(&package, "lib.veac")
        .unwrap_err()
        .message()
        .contains("digest mismatch"));
    assert!(loader
        .load_package(&identity("missing", "1.0.0"), "lib.veac")
        .is_err());
}

#[cfg(unix)]
#[test]
fn package_source_loader_rejects_hard_linked_locked_sources() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    let source = temp.path().join("vendor/util/lib.veac");
    let alias = temp.path().join("vendor/util/alias.veac");
    std::fs::hard_link(&source, &alias).unwrap();

    helpers::lock_dependency_source(temp.path(), "alias.veac", b"module {}\n");

    let error = PackageSourceLoader::for_root(temp.path()).unwrap_err();
    assert!(error.message().contains("same physical file"));
}

#[test]
fn package_source_loader_drives_the_existing_program_resolver() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    let root_source = "module {\n  import \"package:util@2.0.0/lib.veac\" as util;\n  export fn workspace() -> time { util.duration() }\n}\n";
    std::fs::write(temp.path().join("main.veac"), root_source).unwrap();
    let dependency = b"module { export fn duration() -> time { 1s } }\n";
    std::fs::write(temp.path().join("vendor/util/lib.veac"), dependency).unwrap();
    helpers::rewrite_entry_digest(temp.path(), root_source.as_bytes());
    helpers::rewrite_dependency_digest(temp.path(), dependency);
    let dependency_api = helpers::derive_api(
        temp.path().join("vendor/util/lib.veac"),
        identity("util", "2.0.0"),
    );
    helpers::rewrite_dependency_api(temp.path(), &dependency_api);
    let derivation = root_source.replace("package:util@2.0.0/lib.veac", "vendor/util/lib.veac");
    std::fs::write(temp.path().join("main.veac"), &derivation).unwrap();
    let root_api = helpers::derive_root_api(temp.path());
    std::fs::write(temp.path().join("main.veac"), root_source).unwrap();
    helpers::rewrite_root_api(temp.path(), &root_api);
    let (loader, entry) = PackageSourceLoader::for_root(temp.path()).unwrap();
    let interface = CompilerDatabase::default()
        .module_interface(entry, &loader)
        .unwrap();
    assert_eq!(interface.functions[0].name, "workspace");
}

#[path = "package_contract_helpers.rs"]
mod helpers;
#[path = "package_contract/interface_verification.rs"]
mod interface_verification;
