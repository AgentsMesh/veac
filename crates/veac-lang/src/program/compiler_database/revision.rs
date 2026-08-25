use sha2::{Digest, Sha256};

const DOMAIN: &[u8] = b"veac.source-revision-query.v1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompilerSourceRevision([u8; 32]);

impl CompilerSourceRevision {
    pub fn sha256(self) -> String {
        let mut output = String::with_capacity(64);
        for byte in self.0 {
            use std::fmt::Write;
            write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
        }
        output
    }

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
        Self(digest.finalize().into())
    }
}

fn frame(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}
