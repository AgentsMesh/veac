use sha2::{Digest, Sha256};

const DOMAIN: &[u8] = b"veac.entity-id.v1\0";

pub(super) fn project(path: &[&str]) -> veac_ir::ProjectId {
    veac_ir::ProjectId::from_digest(stable("project", path))
}

pub(in crate::program::executable) fn sequence(path: &[&str]) -> veac_ir::SequenceId {
    veac_ir::SequenceId::from_digest(stable("sequence", path))
}

pub(super) fn track(path: &[&str]) -> veac_ir::TrackId {
    veac_ir::TrackId::from_digest(stable("track", path))
}

pub(in crate::program::executable) fn item(path: &[&str]) -> veac_ir::ItemId {
    veac_ir::ItemId::from_digest(stable("item", path))
}

pub(super) fn relation(path: &[&str]) -> veac_ir::RelationId {
    veac_ir::RelationId::from_digest(stable("relation", path))
}

pub(in crate::program::executable) fn material(path: &[&str]) -> veac_ir::MaterialId {
    veac_ir::MaterialId::from_digest(stable("material", path))
}

pub(super) fn multicam(path: &[&str]) -> veac_ir::MulticamGroupId {
    veac_ir::MulticamGroupId::from_digest(stable("multicam", path))
}

pub(super) fn angle(path: &[&str]) -> veac_ir::MulticamAngleId {
    veac_ir::MulticamAngleId::from_digest(stable("multicam-angle", path))
}

pub(super) fn annotation(path: &[&str]) -> veac_ir::AnnotationId {
    veac_ir::AnnotationId::from_digest(stable("annotation", path))
}

pub(in crate::program::executable) fn apply(path: &[&str]) -> veac_ir::ApplyId {
    veac_ir::ApplyId::from_digest(stable("apply", path))
}

pub(in crate::program::executable) fn apply_stage(path: &[&str]) -> veac_ir::ApplyStageId {
    veac_ir::ApplyStageId::from_digest(stable("apply-stage", path))
}

pub(super) fn delivery(path: &[&str]) -> veac_ir::RenderConfigId {
    veac_ir::RenderConfigId::from_digest(stable("delivery", path))
}

pub(in crate::program::executable) fn deliverable(path: &[&str]) -> veac_ir::DeliverableId {
    veac_ir::DeliverableId::from_digest(stable("deliverable", path))
}

pub(super) fn rendition(path: &[&str]) -> veac_ir::HlsRenditionId {
    veac_ir::HlsRenditionId::from_digest(stable("hls-rendition", path))
}

pub(super) fn keyframe(path: &[&str]) -> veac_ir::KeyframeId {
    veac_ir::KeyframeId::from_digest(stable("keyframe", path))
}

pub(super) fn audio_processor(path: &[&str]) -> veac_ir::AudioProcessorId {
    veac_ir::AudioProcessorId::from_digest(stable("audio-processor", path))
}

pub(super) fn eq_band(path: &[&str]) -> veac_ir::EqBandId {
    veac_ir::EqBandId::from_digest(stable("eq-band", path))
}

pub(super) fn stable(kind: &str, path: &[&str]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(DOMAIN);
    frame(&mut digest, kind.as_bytes());
    digest.update(encoded_len(path.len()).to_be_bytes());
    for segment in path {
        frame(&mut digest, segment.as_bytes());
    }
    digest.finalize().into()
}

pub(in crate::program::executable) fn effect(path: &[&str]) -> veac_ir::EffectId {
    veac_ir::EffectId::from_digest(stable("effect", path))
}

fn frame(digest: &mut Sha256, value: &[u8]) {
    digest.update(encoded_len(value.len()).to_be_bytes());
    digest.update(value);
}

fn encoded_len(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}
