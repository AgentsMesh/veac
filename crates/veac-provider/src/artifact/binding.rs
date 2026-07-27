use veac_artifact::{artifact_key, ArtifactDependency, ContentDigest};

use crate::{ProviderResponseEnvelope, ProviderResult, Validate};

pub const PROVIDER_EXECUTABLE_DEPENDENCY_ROLE: &str = "provider_executable";

pub fn bind_provider_executable(
    response: &ProviderResponseEnvelope,
    executable: ContentDigest,
) -> ProviderResult<ProviderResponseEnvelope> {
    response.validate()?;
    executable.validate()?;
    let mut bound = response.clone();
    for artifact in bound.output.artifacts_mut() {
        if artifact
            .descriptor
            .dependencies
            .iter()
            .any(|dependency| dependency.role == PROVIDER_EXECUTABLE_DEPENDENCY_ROLE)
        {
            return crate::validation::invalid(
                "provider artifact declares the reserved provider_executable dependency",
            );
        }
        artifact.descriptor.dependencies.push(ArtifactDependency {
            role: PROVIDER_EXECUTABLE_DEPENDENCY_ROLE.to_owned(),
            identity: executable.clone(),
        });
        artifact.descriptor.dependencies.sort_by(|left, right| {
            (&left.role, &left.identity.value).cmp(&(&right.role, &right.identity.value))
        });
        artifact.record.key = artifact_key(&artifact.descriptor)?;
    }
    bound.validate()?;
    Ok(bound)
}
