use sha2::{Digest, Sha256};

use crate::vocabulary::PluginEffectSpec;

pub(super) fn update(digest: &mut Sha256, values: &[PluginEffectSpec]) {
    digest.update((values.len() as u64).to_be_bytes());
    for value in values {
        super::framed(digest, value.effect_type.as_bytes());
        super::framed(digest, value.digest.as_bytes());
    }
}
