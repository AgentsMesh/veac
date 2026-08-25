use sha2::{Digest, Sha256};
use std::sync::Arc;

const DOMAIN: &[u8] = b"veac.syntax-query.v1\0";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct SyntaxQueryKey {
    source_id: String,
    source: Arc<str>,
    digest: [u8; 32],
}

impl SyntaxQueryKey {
    pub(super) fn new(source_id: &str, source: &str) -> Self {
        let mut digest = Sha256::new();
        digest.update(DOMAIN);
        frame(&mut digest, env!("CARGO_PKG_VERSION").as_bytes());
        frame(
            &mut digest,
            &crate::vocabulary::LANGUAGE_SPEC_SCHEMA_VERSION.to_be_bytes(),
        );
        frame(&mut digest, source_id.as_bytes());
        frame(&mut digest, source.as_bytes());
        Self {
            source_id: source_id.to_owned(),
            source: Arc::from(source),
            digest: digest.finalize().into(),
        }
    }

    pub(super) fn retained_bytes(&self) -> usize {
        self.source_id
            .len()
            .saturating_mul(2)
            .saturating_add(self.source.len())
    }
}

fn frame(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}
