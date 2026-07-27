use sha2::{Digest, Sha256};
use veac_ir::MediaIdentity;

use super::{InputBinding, MediaRole};
use crate::{
    canonical_descriptor_bytes, ArtifactDescriptor, ArtifactRecord, ContentDigest, DigestAlgorithm,
};

pub(super) fn original_resource(identity: &MediaIdentity) -> ContentDigest {
    let mut bytes = b"veac-original-resource-v1\0".to_vec();
    field(&mut bytes, identity.digest.as_bytes());
    ContentDigest::sha256(bytes)
}

pub(super) fn artifact_resource(
    descriptor: &ArtifactDescriptor,
    record: &ArtifactRecord,
) -> ContentDigest {
    let mut bytes = b"veac-artifact-resource-v1\0".to_vec();
    let descriptor = canonical_descriptor_bytes(descriptor)
        .expect("a verified artifact always contains a valid canonical descriptor");
    field(&mut bytes, &descriptor);
    field(&mut bytes, record.key.value.as_bytes());
    field(&mut bytes, record.content.value.as_bytes());
    bytes.extend_from_slice(&record.size_bytes.to_be_bytes());
    ContentDigest::sha256(bytes)
}

pub(super) fn execution<'a>(
    inputs: impl Iterator<Item = (&'a veac_plan::PlanInputId, &'a InputBinding)>,
    segment: Option<&super::BoundResource>,
) -> ContentDigest {
    let mut digest = Sha256::new();
    digest.update(b"veac-execution-substitution-v1\0");
    for (id, input) in inputs {
        update_field(&mut digest, id.as_str().as_bytes());
        if let Some(resource) = input.resource() {
            update_field(&mut digest, b"resource");
            update_resource(&mut digest, resource);
        }
        for (role, stream) in [
            (MediaRole::Video, input.video()),
            (MediaRole::Audio, input.audio()),
        ] {
            let Some(stream) = stream else {
                continue;
            };
            update_field(&mut digest, role_name(role));
            update_resource(&mut digest, stream.resource());
            digest.update(stream.physical_stream().global_index.to_be_bytes());
            digest.update(stream.physical_stream().type_index.to_be_bytes());
            let clock = stream.clock();
            digest.update(clock.timescale().to_be_bytes());
            digest.update(clock.physical_start().value.to_be_bytes());
            if let Some(range) = clock.logical_range() {
                digest.update([1]);
                digest.update(range.start.value.to_be_bytes());
                digest.update(range.duration.value.to_be_bytes());
            } else {
                digest.update([0]);
            }
        }
    }
    if let Some(resource) = segment {
        update_field(&mut digest, b"full_render_segment");
        update_resource(&mut digest, resource);
    }
    ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: format!("{:x}", digest.finalize()),
    }
}

fn update_resource(digest: &mut Sha256, resource: &super::BoundResource) {
    update_field(digest, resource.identity().digest.as_bytes());
    update_field(digest, resource.proof_digest().value.as_bytes());
    if let Some((key, content, size)) = resource.proof_fields() {
        update_field(digest, key.value.as_bytes());
        update_field(digest, content.value.as_bytes());
        digest.update(size.to_be_bytes());
    }
}

fn role_name(role: MediaRole) -> &'static [u8] {
    match role {
        MediaRole::Video => b"video",
        MediaRole::Audio => b"audio",
    }
}

fn update_field(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn field(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(&(value.len() as u64).to_be_bytes());
    output.extend_from_slice(value);
}
