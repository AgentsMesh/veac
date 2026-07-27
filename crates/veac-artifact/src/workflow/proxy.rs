use crate::{
    artifact_key, ArtifactDescriptor, ArtifactError, ArtifactErrorKind, ArtifactKind,
    ArtifactResult, ArtifactStore, ContentDigest, VerifiedArtifact,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ProxySelectionRequest {
    pub source_identity: ContentDigest,
    pub video: Option<ArtifactDescriptor>,
    pub audio: Option<ArtifactDescriptor>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProxyBinding {
    pub source_identity: ContentDigest,
    pub video: Option<VerifiedArtifact>,
    pub audio: Option<VerifiedArtifact>,
}

pub fn select_proxy(
    store: &ArtifactStore,
    request: &ProxySelectionRequest,
) -> ArtifactResult<ProxyBinding> {
    select_proxy_while(store, request, || true)
}

pub fn select_proxy_while(
    store: &ArtifactStore,
    request: &ProxySelectionRequest,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<ProxyBinding> {
    active(&mut guard)?;
    request.source_identity.validate()?;
    if request.video.is_none() && request.audio.is_none() {
        return invalid("proxy selection requires a video or audio descriptor");
    }
    validate_descriptor(
        request.video.as_ref(),
        ArtifactKind::ProxyVideo,
        &request.source_identity,
    )?;
    validate_descriptor(
        request.audio.as_ref(),
        ArtifactKind::ProxyAudio,
        &request.source_identity,
    )?;
    let video = load(store, request.video.as_ref(), &mut guard)?;
    let audio = load(store, request.audio.as_ref(), &mut guard)?;
    Ok(ProxyBinding {
        source_identity: request.source_identity.clone(),
        video,
        audio,
    })
}

fn active(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    if guard() {
        Ok(())
    } else {
        Err(ArtifactError::new(
            ArtifactErrorKind::ResourceLimit,
            "proxy selection exceeded its caller resource guard",
        ))
    }
}

fn load(
    store: &ArtifactStore,
    descriptor: Option<&ArtifactDescriptor>,
    guard: impl FnMut() -> bool,
) -> ArtifactResult<Option<VerifiedArtifact>> {
    let Some(descriptor) = descriptor else {
        return Ok(None);
    };
    let key = artifact_key(descriptor)?;
    store.open_verified_bounded_while(&key, descriptor, crate::MAX_ARTIFACT_PAYLOAD_BYTES, guard)
}

fn validate_descriptor(
    descriptor: Option<&ArtifactDescriptor>,
    kind: ArtifactKind,
    source: &ContentDigest,
) -> ArtifactResult<()> {
    let Some(descriptor) = descriptor else {
        return Ok(());
    };
    descriptor.validate()?;
    let inputs: Vec<_> = descriptor
        .dependencies
        .iter()
        .filter(|dependency| dependency.role == "input")
        .collect();
    if descriptor.kind != kind || inputs.len() != 1 || inputs[0].identity != *source {
        return invalid("proxy descriptor is not bound to the requested source identity");
    }
    Ok(())
}

fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}

#[cfg(test)]
#[path = "proxy/guard_tests.rs"]
mod guard_tests;
