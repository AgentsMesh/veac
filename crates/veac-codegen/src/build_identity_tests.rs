use super::build_identity_contract::{
    build_fingerprint, source_fingerprint, BuildInputs, SourceFile,
};

const DOMAIN: &str = "veac.codegen";

#[test]
fn source_inventory_is_order_independent_but_path_and_bytes_are_exact() {
    let files = [
        SourceFile {
            path: "src/a.rs",
            bytes: b"a",
        },
        SourceFile {
            path: "src/b.rs",
            bytes: b"b",
        },
    ];
    let reversed = [files[1], files[0]];
    let expected = source_fingerprint(DOMAIN, &files).unwrap();
    assert_eq!(source_fingerprint(DOMAIN, &reversed).unwrap(), expected);
    let changed_path = [
        files[0],
        SourceFile {
            path: "src/c.rs",
            bytes: b"b",
        },
    ];
    let changed_bytes = [
        files[0],
        SourceFile {
            path: "src/b.rs",
            bytes: b"c",
        },
    ];
    assert_ne!(source_fingerprint(DOMAIN, &changed_path).unwrap(), expected);
    assert_ne!(
        source_fingerprint(DOMAIN, &changed_bytes).unwrap(),
        expected
    );
    assert!(source_fingerprint(DOMAIN, &[files[0], files[0]]).is_err());
}

#[test]
fn build_identity_binds_toolchain_target_features_and_version() {
    let source = "ab".repeat(32);
    let features = vec!["B".into(), "A".into()];
    let expected =
        build_fingerprint(DOMAIN, &input(&source, "rustc", "target", "1", &features)).unwrap();
    let reordered = vec!["A".into(), "B".into(), "A".into()];
    assert_eq!(
        build_fingerprint(DOMAIN, &input(&source, "rustc", "target", "1", &reordered),).unwrap(),
        expected
    );
    for changed in [
        input(&source, "other", "target", "1", &features),
        input(&source, "rustc", "other", "1", &features),
        input(&source, "rustc", "target", "2", &features),
        input(&source, "rustc", "target", "1", &[]),
    ] {
        assert_ne!(build_fingerprint(DOMAIN, &changed).unwrap(), expected);
    }
}

fn input<'a>(
    source: &'a str,
    rustc: &'a str,
    target: &'a str,
    version: &'a str,
    features: &'a [String],
) -> BuildInputs<'a> {
    BuildInputs {
        source_sha256: source,
        rustc_verbose: rustc,
        target,
        features,
        package_version: version,
    }
}
