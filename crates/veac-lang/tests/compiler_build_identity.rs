#[path = "../build_identity.rs"]
mod build_identity;

use build_identity::{BuildInputs, SourceFile};

fn source(path: &str, bytes: &[u8]) -> String {
    build_identity::source_fingerprint(&[SourceFile { path, bytes }]).unwrap()
}

fn build<'a>(
    source_sha256: &'a str,
    rustc_verbose: &'a str,
    target: &'a str,
    features: &'a [String],
) -> BuildInputs<'a> {
    BuildInputs {
        source_sha256,
        rustc_verbose,
        target,
        features,
        package_version: "0.1.0",
    }
}

#[test]
fn source_identity_is_sorted_deterministic_and_content_addressed() {
    let left = SourceFile {
        path: "crates/veac-lang/src/lib.rs",
        bytes: b"pub fn one() {}",
    };
    let right = SourceFile {
        path: "crates/veac-ir/src/lib.rs",
        bytes: b"pub struct Ir;",
    };
    let forward = build_identity::source_fingerprint(&[left, right]).unwrap();
    let reverse = build_identity::source_fingerprint(&[right, left]).unwrap();
    assert_eq!(forward, reverse);
    assert_ne!(source(left.path, b"pub fn two() {}"), forward);
    assert_ne!(source("crates/veac-lang/src/other.rs", left.bytes), forward);
}

#[test]
fn source_identity_rejects_absolute_duplicate_and_unnormalized_paths() {
    for path in ["/tmp/src/lib.rs", "../src/lib.rs", "src//lib.rs"] {
        assert!(build_identity::source_fingerprint(&[SourceFile {
            path,
            bytes: b"same",
        }])
        .is_err());
    }
    let duplicate = SourceFile {
        path: "src/lib.rs",
        bytes: b"same",
    };
    assert!(build_identity::source_fingerprint(&[duplicate, duplicate]).is_err());
}

#[test]
fn build_identity_covers_toolchain_target_features_and_source() {
    let source_a = source("src/lib.rs", b"a");
    let source_b = source("src/lib.rs", b"b");
    let no_features = Vec::new();
    let feature = vec!["CARGO_FEATURE_EXACT".to_owned()];
    let baseline = build_identity::build_fingerprint(&build(
        &source_a,
        "rustc 1.90.0\ncommit-hash: abc\n",
        "aarch64-apple-darwin",
        &no_features,
    ))
    .unwrap();
    for changed in [
        build(
            &source_b,
            "rustc 1.90.0\ncommit-hash: abc\n",
            "aarch64-apple-darwin",
            &no_features,
        ),
        build(
            &source_a,
            "rustc 1.91.0\ncommit-hash: def\n",
            "aarch64-apple-darwin",
            &no_features,
        ),
        build(
            &source_a,
            "rustc 1.90.0\ncommit-hash: abc\n",
            "x86_64-unknown-linux-gnu",
            &no_features,
        ),
        build(
            &source_a,
            "rustc 1.90.0\ncommit-hash: abc\n",
            "aarch64-apple-darwin",
            &feature,
        ),
    ] {
        assert_ne!(
            build_identity::build_fingerprint(&changed).unwrap(),
            baseline
        );
    }
}

#[test]
fn build_identity_excludes_workspace_path_clock_and_feature_order() {
    let digest = source("src/lib.rs", b"same source");
    let first = vec!["CARGO_FEATURE_B".to_owned(), "CARGO_FEATURE_A".to_owned()];
    let second = vec!["CARGO_FEATURE_A".to_owned(), "CARGO_FEATURE_B".to_owned()];
    let left = build_identity::build_fingerprint(&build(&digest, "rustc", "target", &first));
    let right = build_identity::build_fingerprint(&build(&digest, "rustc", "target", &second));
    assert_eq!(left.unwrap(), right.unwrap());
}

#[test]
fn build_identity_rejects_noncanonical_source_digests() {
    let features = Vec::new();
    for digest in ["", &"A".repeat(64), &"g".repeat(64), &"0".repeat(63)] {
        let error = build_identity::build_fingerprint(&build(digest, "rustc", "target", &features))
            .unwrap_err();
        assert!(error.contains("lowercase SHA-256"));
    }
}
