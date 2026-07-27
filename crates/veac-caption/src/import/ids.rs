use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use subtitler::model::Subtitle;

use crate::{CaptionCueId, CaptionFormat};

pub(super) struct IdFactory {
    namespace: String,
    occurrences: BTreeMap<String, usize>,
}

impl IdFactory {
    pub(super) fn new(namespace: &str) -> Self {
        Self {
            namespace: namespace.to_owned(),
            occurrences: BTreeMap::new(),
        }
    }

    pub(super) fn next(
        &mut self,
        format: CaptionFormat,
        native_id: Option<&str>,
        subtitle: &Subtitle,
    ) -> CaptionCueId {
        let identity = native_id.map_or_else(
            || {
                format!(
                    "{}:{}:{}",
                    subtitle.start,
                    subtitle.end,
                    subtitle.plaintext()
                )
            },
            ToOwned::to_owned,
        );
        let seed = format!("{}|{format}|{identity}", self.namespace);
        let occurrence = self.occurrences.entry(seed.clone()).or_default();
        let unique = format!("{seed}|{occurrence}");
        *occurrence += 1;
        let digest = Sha256::digest(unique.as_bytes());
        let suffix: String = digest[..16]
            .iter()
            .flat_map(|byte| [hex(byte >> 4), hex(byte & 0x0f)])
            .collect();
        CaptionCueId::new(format!("cap_{suffix}")).expect("digest is a valid cue ID")
    }
}

fn hex(value: u8) -> char {
    char::from(b"0123456789abcdef"[usize::from(value)])
}
