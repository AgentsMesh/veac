use std::fs;
use std::path::{Path, PathBuf};
use veac_lang::package::api::{
    canonical_api_metadata_json, from_module_interface, package_api_digest,
};
use veac_lang::package::{
    canonical_package_lock_json, canonical_package_manifest_json, package_content_digest,
    sha256_bytes, ExactVersion, LockedFile, PackageIdentity, PackageLockV1, PackageManifestV1,
    PackageName,
};
use veac_lang::program::{
    CompilerDatabase, FileSystemLoader, LoadedSource, SourceAuthority, SourceLoader,
};

const SOURCE_PREFIX: &str = "packages/veac-components@0.1.0/";

struct PackageNamespaceLoader(FileSystemLoader);

impl SourceLoader for PackageNamespaceLoader {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let internal = importer
            .strip_prefix(SOURCE_PREFIX)
            .ok_or_else(|| format!("unexpected package source ID {importer}"))?;
        let mut loaded = self.0.load(internal, requested)?;
        loaded.id = format!("{SOURCE_PREFIX}{}", loaded.id);
        Ok(loaded)
    }

    fn authority(&self, _: &str) -> SourceAuthority {
        SourceAuthority::ReadOnlyDependency
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/veac-components")
}
fn identity() -> PackageIdentity {
    PackageIdentity {
        name: PackageName::new("veac-components").unwrap(),
        version: ExactVersion::new("0.1.0").unwrap(),
    }
}

#[test]
fn generate_contract_files() {
    let root = root();
    let package = identity();
    let entry_path = root.join("main.veac");
    let (loader, mut entry) = FileSystemLoader::for_entry(&entry_path).unwrap();
    entry.id = format!("{SOURCE_PREFIX}{}", entry.id);
    let interface = CompilerDatabase::default()
        .module_interface(entry, &PackageNamespaceLoader(loader))
        .unwrap();
    let api = from_module_interface(package.clone(), &interface);
    let api_json = canonical_api_metadata_json(&api).unwrap();
    fs::write(root.join("veac.package.api.json"), format!("{api_json}\n")).unwrap();
    let paths = [
        "main.veac",
        "layout.veac",
        "text.veac",
        "motion.veac",
        "media.veac",
        "audio.veac",
        "delivery.veac",
        "components/card.veac",
    ];
    let mut files = paths
        .into_iter()
        .map(|relative| (root.join(relative), relative.to_owned()))
        .map(|path| {
            let (path, relative) = path;
            let bytes = fs::read(path).unwrap();
            LockedFile {
                path: relative,
                sha256: sha256_bytes(&bytes),
            }
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let manifest = PackageManifestV1::new(
        package.clone(),
        "main.veac",
        files
            .iter()
            .find(|file| file.path == "main.veac")
            .unwrap()
            .sha256
            .clone(),
        vec![],
        package_api_digest(&api).unwrap(),
    );
    let lock = PackageLockV1::new(package, files.clone(), vec![]).unwrap();
    fs::write(
        root.join("veac.package.json"),
        format!("{}\n", canonical_package_manifest_json(&manifest).unwrap()),
    )
    .unwrap();
    fs::write(
        root.join("veac.package.lock"),
        format!("{}\n", canonical_package_lock_json(&lock).unwrap()),
    )
    .unwrap();
    assert_eq!(
        lock.root_content_sha256,
        package_content_digest(&files).unwrap()
    );
}
