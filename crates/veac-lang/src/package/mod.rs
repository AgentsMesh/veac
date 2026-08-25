//! Versioned, content-addressed package contracts and local discovery.

mod content;
mod discovery;
mod error;
mod fs;
mod graph;
mod interface_verification;
mod json;
mod model;
mod mount_set;
pub(crate) mod portable_contract;
pub(crate) mod source_id;
mod source_loader;
mod source_path;
mod source_registry;
mod validation;
mod value;

pub mod api;

pub use content::{package_content_digest, sha256_bytes};
pub use discovery::{discover_package, DiscoveredDependency, DiscoveredPackage, PackageDiscovery};
pub use error::{PackageError, PackageErrorKind};
pub use json::{
    canonical_package_lock_json, canonical_package_manifest_json, package_lock_json_schema,
    package_manifest_json_schema, parse_package_lock_json, parse_package_manifest_json,
};
pub use model::{
    LockedFile, LockedPackage, PackageCompatibility, PackageDependency, PackageLockV1,
    PackageManifestV1,
};
pub use mount_set::VerifiedPackageMountSet;
pub use source_loader::PackageSourceLoader;
pub use value::{ExactVersion, PackageIdentity, PackageName, Sha256Digest};

pub const PACKAGE_MANIFEST_FILE: &str = "veac.package.json";
pub const PACKAGE_LOCK_FILE: &str = "veac.package.lock";
pub const PACKAGE_API_FILE: &str = "veac.package.api.json";
pub const PACKAGE_MANIFEST_SCHEMA: &str = "https://veac.dev/schemas/package-manifest/v1";
pub const PACKAGE_LOCK_SCHEMA: &str = "https://veac.dev/schemas/package-lock/v1";
pub const PACKAGE_SCHEMA_VERSION: u32 = 1;
pub const MAX_PACKAGE_JSON_BYTES: usize = 1024 * 1024;
pub const MAX_LOCKED_PACKAGES: usize = 4096;
pub const MAX_PACKAGE_FILES: usize = 4096;
pub const MAX_PACKAGE_FILE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_PACKAGE_CONTENT_BYTES: usize = 64 * 1024 * 1024;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
