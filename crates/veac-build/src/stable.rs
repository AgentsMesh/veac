use veac_artifact::ContentDigest;

#[derive(Default)]
pub(crate) struct StableBytes(Vec<u8>);

impl StableBytes {
    pub fn text(&mut self, value: &str) {
        self.bytes(value.as_bytes());
    }

    pub fn bytes(&mut self, value: &[u8]) {
        self.0
            .extend_from_slice(&(value.len() as u64).to_be_bytes());
        self.0.extend_from_slice(value);
    }

    pub fn u16(&mut self, value: u16) {
        self.0.extend_from_slice(&value.to_be_bytes());
    }

    pub fn u32(&mut self, value: u32) {
        self.0.extend_from_slice(&value.to_be_bytes());
    }

    pub fn digest(self) -> ContentDigest {
        ContentDigest::sha256(self.0)
    }
}
