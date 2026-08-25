use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ExactVersion, PackageIdentity, PackageName, Sha256Digest};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackageManifestV1 {
    #[schemars(extend("const" = super::PACKAGE_MANIFEST_SCHEMA))]
    pub schema: String,
    #[schemars(extend("const" = super::PACKAGE_SCHEMA_VERSION))]
    pub schema_version: u32,
    pub package: PackageIdentity,
    pub entry: String,
    pub entry_sha256: Sha256Digest,
    pub dependencies: Vec<PackageDependency>,
    pub api_sha256: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackageDependency {
    pub name: PackageName,
    pub version: ExactVersion,
}

impl PackageDependency {
    pub fn identity(&self) -> PackageIdentity {
        PackageIdentity {
            name: self.name.clone(),
            version: self.version.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackageLockV1 {
    #[schemars(extend("const" = super::PACKAGE_LOCK_SCHEMA))]
    pub schema: String,
    #[schemars(extend("const" = super::PACKAGE_SCHEMA_VERSION))]
    pub schema_version: u32,
    pub root: PackageIdentity,
    pub compatibility: PackageCompatibility,
    pub root_files: Vec<LockedFile>,
    pub root_content_sha256: Sha256Digest,
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackageCompatibility {
    pub language_version: ExactVersion,
    pub core_version: u16,
    pub domain_opset_version: u16,
    pub canonical_schema_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LockedPackage {
    pub package: PackageIdentity,
    pub path: String,
    pub entry: String,
    pub dependencies: Vec<PackageDependency>,
    pub files: Vec<LockedFile>,
    pub content_sha256: Sha256Digest,
    pub api_sha256: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LockedFile {
    pub path: String,
    pub sha256: Sha256Digest,
}

impl PackageManifestV1 {
    pub fn new(
        package: PackageIdentity,
        entry: impl Into<String>,
        entry_sha256: Sha256Digest,
        dependencies: Vec<PackageDependency>,
        api_sha256: Sha256Digest,
    ) -> Self {
        Self {
            schema: super::PACKAGE_MANIFEST_SCHEMA.to_owned(),
            schema_version: super::PACKAGE_SCHEMA_VERSION,
            package,
            entry: entry.into(),
            entry_sha256,
            dependencies,
            api_sha256,
        }
    }

    pub fn validate(&self) -> Result<(), super::PackageError> {
        super::validation::manifest(self)
    }
}

impl PackageLockV1 {
    pub fn new(
        root: PackageIdentity,
        root_files: Vec<LockedFile>,
        packages: Vec<LockedPackage>,
    ) -> Result<Self, super::PackageError> {
        let root_content_sha256 = super::package_content_digest(&root_files)?;
        Self {
            schema: super::PACKAGE_LOCK_SCHEMA.to_owned(),
            schema_version: super::PACKAGE_SCHEMA_VERSION,
            root,
            compatibility: PackageCompatibility::current(),
            root_files,
            root_content_sha256,
            packages,
        }
        .validated()
    }

    pub fn validate(&self) -> Result<(), super::PackageError> {
        super::validation::lock(self)
    }

    fn validated(self) -> Result<Self, super::PackageError> {
        self.validate()?;
        Ok(self)
    }
}

impl PackageCompatibility {
    pub fn current() -> Self {
        Self {
            language_version: ExactVersion::new(env!("CARGO_PKG_VERSION"))
                .expect("crate version is exact SemVer"),
            core_version: crate::program::expression::CORE_VERSION,
            domain_opset_version: crate::program::DomainOpsetVersion::CURRENT.raw(),
            canonical_schema_version: veac_ir::CURRENT_SCHEMA_VERSION,
        }
    }
}
