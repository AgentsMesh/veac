use std::path::{Component, Path};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ArtifactError, ArtifactErrorKind, ArtifactResult, ContentDigest, MAX_ARTIFACT_METADATA_BYTES,
    MAX_RENDER_TASK_OUTPUT_BYTES,
};

pub const MAX_DELIVERY_PACKAGE_MEMBERS: usize = 4_096;
pub const MAX_DELIVERY_PACKAGE_PATH_BYTES: usize = 240;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryPackageNodeType {
    Directory,
    RegularFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeliveryPackageMember {
    pub path: String,
    pub node_type: DeliveryPackageNodeType,
    pub size_bytes: u64,
    pub content: Option<ContentDigest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeliveryPackageInventory {
    pub entrypoint: String,
    pub members: Vec<DeliveryPackageMember>,
    pub tree: ContentDigest,
}

#[derive(Serialize)]
struct Tree<'a> {
    entrypoint: &'a str,
    members: &'a [DeliveryPackageMember],
}

impl DeliveryPackageInventory {
    pub fn new(
        entrypoint: impl Into<String>,
        members: Vec<DeliveryPackageMember>,
    ) -> ArtifactResult<Self> {
        let entrypoint = entrypoint.into();
        validate_members(&entrypoint, &members)?;
        let tree = tree_digest(&entrypoint, &members)?;
        Ok(Self {
            entrypoint,
            members,
            tree,
        })
    }

    pub fn validate(&self) -> ArtifactResult<()> {
        validate_members(&self.entrypoint, &self.members)?;
        self.tree.validate()?;
        if self.tree != tree_digest(&self.entrypoint, &self.members)? {
            return invalid("delivery package tree digest does not match its inventory");
        }
        Ok(())
    }

    pub fn size_bytes(&self) -> u64 {
        self.members.iter().map(|member| member.size_bytes).sum()
    }
}

fn validate_members(entrypoint: &str, members: &[DeliveryPackageMember]) -> ArtifactResult<()> {
    if members.is_empty() || members.len() > MAX_DELIVERY_PACKAGE_MEMBERS {
        return limit("delivery package member count exceeds its budget");
    }
    if !safe_path(entrypoint) {
        return invalid("delivery package entrypoint must be a safe relative path");
    }
    let mut previous = None;
    let mut total = 0_u64;
    let mut found_entrypoint = false;
    for member in members {
        if !safe_path(&member.path) || previous.is_some_and(|value| value >= member.path.as_str()) {
            return invalid("delivery package members must have sorted unique safe paths");
        }
        member_shape(member)?;
        total = total
            .checked_add(member.size_bytes)
            .ok_or_else(|| limit_error("delivery package byte size overflow"))?;
        found_entrypoint |=
            member.path == entrypoint && member.node_type == DeliveryPackageNodeType::RegularFile;
        previous = Some(member.path.as_str());
    }
    if !found_entrypoint {
        return invalid("delivery package entrypoint must name a regular file member");
    }
    if total > MAX_RENDER_TASK_OUTPUT_BYTES {
        return limit("delivery package exceeds the render output byte budget");
    }
    Ok(())
}

fn member_shape(member: &DeliveryPackageMember) -> ArtifactResult<()> {
    match (member.node_type, member.size_bytes, &member.content) {
        (DeliveryPackageNodeType::Directory, 0, None) => Ok(()),
        (DeliveryPackageNodeType::RegularFile, 1.., Some(content)) => content.validate(),
        _ => invalid("delivery package member shape does not match its node type"),
    }
}

fn tree_digest(
    entrypoint: &str,
    members: &[DeliveryPackageMember],
) -> ArtifactResult<ContentDigest> {
    crate::json::canonical_bounded(
        &Tree {
            entrypoint,
            members,
        },
        MAX_ARTIFACT_METADATA_BYTES,
        "delivery package inventory",
    )
    .map(ContentDigest::sha256)
}

fn safe_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_DELIVERY_PACKAGE_PATH_BYTES
        && !value
            .chars()
            .any(|character| character.is_control() || character == '\\')
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(value) if !value.is_empty()))
}

fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}

fn limit<T>(message: &str) -> ArtifactResult<T> {
    Err(limit_error(message))
}

fn limit_error(message: &str) -> ArtifactError {
    ArtifactError::new(ArtifactErrorKind::ResourceLimit, message)
}
