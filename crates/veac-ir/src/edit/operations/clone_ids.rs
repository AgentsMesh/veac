use sha2::{Digest, Sha256};

use crate::*;

pub(super) fn derived_effect(owner: &ItemId, source: &EffectId) -> EffectId {
    EffectId::new(hash_id("fx", &format!("{owner}:effect:{source}")))
        .expect("hash-derived effect ID is valid")
}

pub(super) fn derived_keyframe(
    owner: &ItemId,
    source: &KeyframeId,
    path: &str,
    role: &str,
) -> KeyframeId {
    KeyframeId::new(hash_id("kf", &format!("{owner}:{path}:{role}:{source}")))
        .expect("hash-derived keyframe ID is valid")
}

pub(super) fn synthetic_keyframe(
    owner: &ItemId,
    path: &str,
    role: &str,
    time: RationalTime,
) -> KeyframeId {
    KeyframeId::new(hash_id(
        "kf",
        &format!("{owner}:{path}:{role}:{}:{}", time.value, time.timescale),
    ))
    .expect("hash-derived keyframe ID is valid")
}

fn hash_id(prefix: &str, seed: &str) -> String {
    let digest = Sha256::digest(seed);
    let mut value = format!("{prefix}_");
    for byte in &digest[..16] {
        value.push_str(&format!("{byte:02x}"));
    }
    value
}
