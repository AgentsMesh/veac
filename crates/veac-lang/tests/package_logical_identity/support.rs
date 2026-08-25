use std::path::Path;

use veac_lang::package::api::{
    canonical_api_metadata_json, from_module_interface, package_api_digest, ApiMetadataV1,
};
use veac_lang::package::{
    canonical_package_lock_json, canonical_package_manifest_json, package_content_digest,
    sha256_bytes, ExactVersion, LockedFile, LockedPackage, PackageDependency, PackageIdentity,
    PackageLockV1, PackageManifestV1, PackageName,
};
use veac_lang::program::{CompilerDatabase, LoadedSource, SourceAuthority, SourceLoader};

pub const UTIL_ID: &str = "packages/util@2.0.0/lib.veac";
pub const UTIL_SOURCE: &str = r#"module {
  export struct Metric { value: int, }
  impl Metric @display {
    export fn doubled(self) -> int { self.value + self.value }
  }
  export fn metric(value: int) -> Metric { Metric { value: value, } }
}
"#;

struct UtilLoader<'a>(&'a str);

impl SourceLoader for UtilLoader<'_> {
    fn load(&self, _: &str, requested: &str) -> Result<LoadedSource, String> {
        if requested != "package:util@2.0.0/lib.veac" {
            return Err(format!("unknown fixture import {requested}"));
        }
        Ok(source(UTIL_ID, self.0))
    }

    fn authority(&self, _: &str) -> SourceAuthority {
        SourceAuthority::ReadOnlyDependency
    }
}

pub fn identity(name: &str, version: &str) -> PackageIdentity {
    PackageIdentity {
        name: PackageName::new(name).unwrap(),
        version: ExactVersion::new(version).unwrap(),
    }
}

pub fn write_util_root(root: &Path) {
    let package = identity("util", "2.0.0");
    let api = derive_api(package.clone(), UTIL_ID, UTIL_SOURCE, UTIL_SOURCE);
    write_root(root, package, UTIL_SOURCE, vec![], api, vec![]);
}

pub fn write_host(root: &Path, name: &str, locator: &str) {
    write_host_with_util(root, name, locator, UTIL_SOURCE);
}

pub fn write_host_with_util(root: &Path, name: &str, locator: &str, util_source: &str) {
    let package = identity(name, "1.0.0");
    let root_id = format!("packages/{name}@1.0.0/main.veac");
    let root_source = r#"module {
  import "package:util@2.0.0/lib.veac" as util;
  export fn doubled(value: int) -> int { util.metric(value).doubled() }
}
"#;
    let util = identity("util", "2.0.0");
    let util_api = derive_api(util.clone(), UTIL_ID, util_source, util_source);
    let dependency = locked_dependency(util.clone(), locator, util_source, &util_api);
    let requirements = vec![requirement(&util)];
    let api = derive_api(package.clone(), &root_id, root_source, util_source);
    std::fs::create_dir_all(root.join(locator)).unwrap();
    std::fs::write(root.join(locator).join("lib.veac"), util_source).unwrap();
    write_api(&root.join(locator), &util_api);
    write_root(
        root,
        package,
        root_source,
        requirements,
        api,
        vec![dependency],
    );
}

fn derive_api(package: PackageIdentity, id: &str, text: &str, util_source: &str) -> ApiMetadataV1 {
    let interface = CompilerDatabase::default()
        .module_interface(source(id, text), &UtilLoader(util_source))
        .unwrap();
    from_module_interface(package, &interface)
}

fn write_root(
    root: &Path,
    package: PackageIdentity,
    text: &str,
    dependencies: Vec<PackageDependency>,
    api: ApiMetadataV1,
    locked: Vec<LockedPackage>,
) {
    let file = LockedFile {
        path: if package.name.as_str() == "util" {
            "lib.veac".into()
        } else {
            "main.veac".into()
        },
        sha256: sha256_bytes(text.as_bytes()),
    };
    let manifest = PackageManifestV1::new(
        package.clone(),
        &file.path,
        file.sha256.clone(),
        dependencies,
        package_api_digest(&api).unwrap(),
    );
    let lock = PackageLockV1::new(package, vec![file.clone()], locked).unwrap();
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(root.join(&file.path), text).unwrap();
    write_api(root, &api);
    std::fs::write(
        root.join("veac.package.json"),
        canonical_package_manifest_json(&manifest).unwrap(),
    )
    .unwrap();
    std::fs::write(
        root.join("veac.package.lock"),
        canonical_package_lock_json(&lock).unwrap(),
    )
    .unwrap();
}

fn locked_dependency(
    package: PackageIdentity,
    path: &str,
    source: &str,
    api: &ApiMetadataV1,
) -> LockedPackage {
    let file = LockedFile {
        path: "lib.veac".into(),
        sha256: sha256_bytes(source.as_bytes()),
    };
    LockedPackage {
        package,
        path: path.into(),
        entry: file.path.clone(),
        dependencies: vec![],
        content_sha256: package_content_digest(std::slice::from_ref(&file)).unwrap(),
        api_sha256: package_api_digest(api).unwrap(),
        files: vec![file],
    }
}

fn requirement(package: &PackageIdentity) -> PackageDependency {
    PackageDependency {
        name: package.name.clone(),
        version: package.version.clone(),
    }
}

fn write_api(root: &Path, api: &ApiMetadataV1) {
    std::fs::write(
        root.join("veac.package.api.json"),
        canonical_api_metadata_json(api).unwrap(),
    )
    .unwrap();
}

fn source(id: &str, text: &str) -> LoadedSource {
    LoadedSource {
        id: id.into(),
        source: text.into(),
    }
}
