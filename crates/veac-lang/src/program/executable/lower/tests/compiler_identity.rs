use super::super::manifest::compiler_for_build;

#[test]
fn executable_compiler_identity_covers_build_and_all_opsets() {
    let baseline = compiler_for_build(&"a".repeat(64), 9, 7, 3);
    assert_eq!(baseline.len(), 64);
    for changed in [
        compiler_for_build(&"b".repeat(64), 9, 7, 3),
        compiler_for_build(&"a".repeat(64), 10, 7, 3),
        compiler_for_build(&"a".repeat(64), 9, 8, 3),
        compiler_for_build(&"a".repeat(64), 9, 7, 4),
    ] {
        assert_ne!(changed, baseline);
    }
}

#[test]
fn embedded_source_and_build_identities_are_lowercase_sha256() {
    for value in [
        env!("VEAC_COMPILER_SOURCE_SHA256"),
        env!("VEAC_COMPILER_BUILD_SHA256"),
    ] {
        assert_eq!(value.len(), 64);
        assert!(value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    }
}
