use serde::Serialize;
use veac_artifact::ContentDigest;
use veac_lang::package::{PackageIdentity, Sha256Digest};
use veac_project::{
    InputId, LocaleId, MatrixAssignment, MediaDerivation, ProfileId, ProjectOutput, ResolvedInput,
    TargetId, TargetInstanceId,
};

use crate::{BuildAction, BuildError, BuildResult};

pub const PROJECT_ACTION_VERSION: u32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectActionKind {
    VeacRender,
    MediaDerivation,
    Evidence,
}

impl ProjectActionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VeacRender => "veac_render",
            Self::MediaDerivation => "media_derivation",
            Self::Evidence => "evidence",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectAction {
    VeacRender {
        computation: ProjectComputation,
        source: ProjectFileSnapshot,
        source_graph: ProjectSourceGraphRevision,
    },
    MediaDerivation {
        computation: ProjectComputation,
        operation: MediaDerivation,
    },
    Evidence {
        computation: ProjectComputation,
        contract: ProjectFileSnapshot,
        source_graph: ProjectSourceGraphRevision,
    },
}

impl ProjectAction {
    pub fn computation(&self) -> &ProjectComputation {
        match self {
            Self::VeacRender { computation, .. }
            | Self::MediaDerivation { computation, .. }
            | Self::Evidence { computation, .. } => computation,
        }
    }

    pub fn kind_name(&self) -> &'static str {
        self.kind().as_str()
    }

    pub fn kind(&self) -> ProjectActionKind {
        match self {
            Self::VeacRender { .. } => ProjectActionKind::VeacRender,
            Self::MediaDerivation { .. } => ProjectActionKind::MediaDerivation,
            Self::Evidence { .. } => ProjectActionKind::Evidence,
        }
    }
}

impl BuildAction for ProjectAction {
    fn kind(&self) -> &str {
        self.kind_name()
    }

    fn version(&self) -> u32 {
        PROJECT_ACTION_VERSION
    }

    fn canonical_bytes(&self) -> BuildResult<Vec<u8>> {
        serde_json_canonicalizer::to_vec(self).map_err(canonical_error)
    }
}

fn canonical_error(error: serde_json::Error) -> BuildError {
    BuildError::invalid(format!("project action cannot be canonicalized: {error}"))
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectComputation {
    pub instance: TargetInstanceId,
    pub target: TargetId,
    pub profile: Option<ProfileId>,
    pub locale: Option<LocaleId>,
    pub matrix: MatrixAssignment,
    pub inputs: Vec<ResolvedInput>,
    pub outputs: Vec<ProjectOutput>,
    pub bound_sources: Vec<ProjectBoundSource>,
    pub package_mounts: Vec<ProjectPackageMountRevision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectFileSnapshot {
    pub path: String,
    pub content: ContentDigest,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectSourceGraphRevision {
    pub root_module: String,
    pub authored_source_graph_sha256: String,
    pub complete_source_graph_sha256: String,
    pub authored_module_count: u32,
    pub authored_modules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectPackageMountRevision {
    pub root: ProjectPackageRevision,
    pub dependencies: Vec<ProjectPackageRevision>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ProjectPackageRevision {
    pub package: PackageIdentity,
    pub entry_module: String,
    pub entry_sha256: Sha256Digest,
    pub content_sha256: Sha256Digest,
    pub api_sha256: Sha256Digest,
    pub dependencies: Vec<PackageIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectBoundSource {
    pub input: InputId,
    pub role: ProjectSourceRole,
    pub snapshot: ProjectFileSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectSourceRole {
    Material,
    AssetFact,
}

#[cfg(test)]
#[path = "action/tests.rs"]
mod tests;
