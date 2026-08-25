use std::path::{Path, PathBuf};

use serde::Serialize;
use veac_lang::package::api::ApiMetadataV1;
use veac_lang::package::{PackageDiscovery, PackageIdentity, Sha256Digest};

#[derive(Serialize)]
pub(super) struct Inspection {
    schema: &'static str,
    schema_version: u32,
    root: PackageSnapshot,
    dependencies: Vec<PackageSnapshot>,
}

#[derive(Serialize)]
pub(super) struct PackageSnapshot {
    package: PackageIdentity,
    path: PathBuf,
    entry: PathBuf,
    entry_sha256: Sha256Digest,
    content_sha256: Sha256Digest,
    api: ApiMetadataV1,
}

#[derive(Serialize)]
pub(super) struct SearchResult<'a> {
    schema: &'static str,
    schema_version: u32,
    store: &'a Path,
    query: &'a str,
    packages: Vec<PackageSnapshot>,
}

impl From<PackageDiscovery> for Inspection {
    fn from(value: PackageDiscovery) -> Self {
        Self {
            schema: "https://veac.dev/schemas/package-inspection/v1",
            schema_version: 1,
            root: PackageSnapshot::from(value.root),
            dependencies: value
                .dependencies
                .into_iter()
                .map(PackageSnapshot::from)
                .collect(),
        }
    }
}

impl From<veac_lang::package::DiscoveredPackage> for PackageSnapshot {
    fn from(value: veac_lang::package::DiscoveredPackage) -> Self {
        Self {
            package: value.package,
            path: value.path,
            entry: value.entry,
            entry_sha256: value.entry_sha256,
            content_sha256: value.content_sha256,
            api: value.api,
        }
    }
}

impl<'a> SearchResult<'a> {
    pub(super) fn new(
        store: &'a Path,
        query: &'a str,
        packages: Vec<veac_lang::package::DiscoveredPackage>,
    ) -> Self {
        Self {
            schema: "https://veac.dev/schemas/package-search/v1",
            schema_version: 1,
            store,
            query,
            packages: packages.into_iter().map(PackageSnapshot::from).collect(),
        }
    }
}
