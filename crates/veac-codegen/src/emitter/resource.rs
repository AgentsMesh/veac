use std::collections::BTreeMap;

use veac_artifact::BoundResource;
use veac_plan::canonical::HashAlgorithm;

use super::error::{diagnostic, CodegenErrorKind};
use super::{BackendResource, CodegenErrors};

pub(super) fn protected<'a>(
    inputs: impl IntoIterator<Item = &'a BoundResource>,
) -> Result<Vec<BackendResource>, CodegenErrors> {
    let mut resources = BTreeMap::<String, BackendResource>::new();
    for input in inputs {
        let path = input.path();
        let Some(path_key) = path.to_str() else {
            return Err(binding_error(
                path.display().to_string(),
                "resource path must be valid UTF-8",
            ));
        };
        if !valid_sha256(input.identity()) {
            return Err(binding_error(
                path.display().to_string(),
                "resource identity must be a lowercase SHA-256 digest",
            ));
        }
        if let Some(existing) = resources.get(path_key) {
            if existing.expected_identity != *input.identity() {
                return Err(binding_error(
                    path.display().to_string(),
                    "one resource path has conflicting expected identities",
                ));
            }
            continue;
        }
        resources.insert(
            path_key.to_owned(),
            BackendResource {
                path: path.to_owned(),
                expected_identity: input.identity().clone(),
            },
        );
    }
    Ok(resources.into_values().collect())
}

fn valid_sha256(identity: &veac_plan::canonical::MediaIdentity) -> bool {
    identity.algorithm == HashAlgorithm::Sha256
        && identity.digest.len() == 64
        && identity
            .digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn binding_error(object_id: String, message: &str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidResourceBinding,
        "RESOURCE_BINDING_INVALID",
        Some(object_id),
        message,
    ))
}
