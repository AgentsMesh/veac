use super::*;

#[test]
fn injected_alias_policies_cover_sensitive_and_folded_future_names() {
    let candidate = PathBuf::from("/output/CAFE\u{301}.JSON");
    let protected = PathBuf::from("/output/Caf\u{e9}.json");
    let peers = vec![&protected];
    let sensitive = Policy {
        case_insensitive: false,
        normalization_insensitive: false,
    };
    let insensitive = Policy {
        case_insensitive: true,
        normalization_insensitive: true,
    };

    assert!(!folded_conflict(&candidate, &peers, sensitive).unwrap());
    assert!(folded_conflict(&candidate, &peers, insensitive).unwrap());
    assert_eq!(fold(&candidate, sensitive).unwrap(), candidate);
    assert_eq!(
        fold(&candidate, insensitive).unwrap(),
        PathBuf::from("/output/caf\u{e9}.json")
    );
}

#[test]
fn missing_parent_maps_alias_probe_failures() {
    let temp = tempfile::tempdir().unwrap();
    let parent = temp.path().join("missing");
    let candidate = parent.join("render.mp4");
    let protected = vec![parent.join("source.mp4")];
    let error = conflicts(&candidate, &protected).unwrap_err();
    assert!(error.to_string().contains("OUTPUT_ALIAS_POLICY"));
}

#[cfg(unix)]
#[test]
fn non_utf8_names_fail_closed_during_alias_folding() {
    use std::os::unix::ffi::OsStringExt;

    let path = PathBuf::from(std::ffi::OsString::from_vec(vec![0xff]));
    let policy = Policy {
        case_insensitive: true,
        normalization_insensitive: true,
    };
    let error = fold(&path, policy).unwrap_err();
    assert!(error.to_string().contains("cannot compare non-UTF-8"));
}
