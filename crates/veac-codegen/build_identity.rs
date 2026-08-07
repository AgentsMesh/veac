use sha2::{Digest, Sha256};

#[derive(Clone, Copy)]
pub struct SourceFile<'a> {
    pub path: &'a str,
    pub bytes: &'a [u8],
}

pub struct BuildInputs<'a> {
    pub source_sha256: &'a str,
    pub rustc_verbose: &'a str,
    pub target: &'a str,
    pub features: &'a [String],
    pub package_version: &'a str,
}

pub fn source_fingerprint(domain: &str, files: &[SourceFile<'_>]) -> Result<String, String> {
    validate_domain(domain)?;
    let mut files = files.iter().collect::<Vec<_>>();
    files.sort_by(|left, right| left.path.cmp(right.path));
    if files.is_empty() {
        return Err("source identity inventory is empty".to_owned());
    }
    let mut digest = Sha256::new();
    header(&mut digest, domain, "source-inventory-v1");
    field(&mut digest, b"count", &(files.len() as u64).to_be_bytes());
    let mut previous = None;
    for file in files {
        validate_path(file.path)?;
        if previous == Some(file.path) {
            return Err(format!("duplicate source path `{}`", file.path));
        }
        field(&mut digest, b"path", file.path.as_bytes());
        field(&mut digest, b"source", file.bytes);
        previous = Some(file.path);
    }
    Ok(hex(digest))
}

pub fn build_fingerprint(domain: &str, input: &BuildInputs<'_>) -> Result<String, String> {
    validate_domain(domain)?;
    validate_sha256(input.source_sha256)?;
    if input.rustc_verbose.trim().is_empty() || input.target.trim().is_empty() {
        return Err("toolchain and target identities must be present".to_owned());
    }
    let mut features = input
        .features
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    features.sort_unstable();
    features.dedup();
    let mut digest = Sha256::new();
    header(&mut digest, domain, "build-v1");
    field(
        &mut digest,
        b"source-sha256",
        input.source_sha256.as_bytes(),
    );
    field(&mut digest, b"rustc-vv", input.rustc_verbose.as_bytes());
    field(&mut digest, b"target", input.target.as_bytes());
    field(
        &mut digest,
        b"package-version",
        input.package_version.as_bytes(),
    );
    field(
        &mut digest,
        b"feature-count",
        &(features.len() as u64).to_be_bytes(),
    );
    for feature in features {
        field(&mut digest, b"feature", feature.as_bytes());
    }
    Ok(hex(digest))
}

fn header(digest: &mut Sha256, domain: &str, kind: &str) {
    digest.update(b"veac.exact-build-identity.v1\0");
    field(digest, b"domain", domain.as_bytes());
    field(digest, b"kind", kind.as_bytes());
}

fn validate_domain(value: &str) -> Result<(), String> {
    (!value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.'))
    .then_some(())
    .ok_or_else(|| "build identity domain is invalid".to_owned())
}

fn validate_path(path: &str) -> Result<(), String> {
    let valid = !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path
            .split('/')
            .any(|part| part.is_empty() || part == ".." || part == ".");
    valid
        .then_some(())
        .ok_or_else(|| format!("source path `{path}` is not normalized and relative"))
}

fn validate_sha256(value: &str) -> Result<(), String> {
    let valid = value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    valid
        .then_some(())
        .ok_or_else(|| "source identity is not lowercase SHA-256".to_owned())
}

fn field(digest: &mut Sha256, name: &[u8], value: &[u8]) {
    digest.update((name.len() as u64).to_be_bytes());
    digest.update(name);
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn hex(digest: Sha256) -> String {
    format!("{:x}", digest.finalize())
}
