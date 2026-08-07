use sha2::{Digest, Sha256};
use veac_artifact::ContentDigest;

pub const RUNTIME_SOURCE_SHA256: &str = env!("VEAC_RUNTIME_SOURCE_SHA256");
pub const RUNTIME_BUILD_SHA256: &str = env!("VEAC_RUNTIME_BUILD_SHA256");

pub fn runtime_backend_identity() -> ContentDigest {
    runtime_identity(RUNTIME_SOURCE_SHA256, RUNTIME_BUILD_SHA256)
}

pub fn artifact_backend_identity() -> ContentDigest {
    combined_identity(
        &veac_codegen::render_implementation_identity().digest,
        &runtime_backend_identity(),
    )
}

fn runtime_identity(source: &str, build: &str) -> ContentDigest {
    let mut digest = Sha256::new();
    digest.update(b"veac.runtime-backend.v1\0");
    field(&mut digest, b"source", source.as_bytes());
    field(&mut digest, b"build", build.as_bytes());
    ContentDigest::sha256(digest.finalize())
}

fn combined_identity(codegen: &ContentDigest, runtime: &ContentDigest) -> ContentDigest {
    let mut digest = Sha256::new();
    digest.update(b"veac.artifact-backend.v1\0");
    field(&mut digest, b"codegen", codegen.value.as_bytes());
    field(&mut digest, b"runtime", runtime.value.as_bytes());
    ContentDigest::sha256(digest.finalize())
}

fn field(digest: &mut Sha256, name: &[u8], value: &[u8]) {
    digest.update((name.len() as u64).to_be_bytes());
    digest.update(name);
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

#[cfg(test)]
#[path = "identity/tests.rs"]
mod tests;
