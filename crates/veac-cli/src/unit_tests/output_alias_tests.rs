use std::path::Path;

#[cfg(unix)]
#[test]
fn guarded_outputs_reject_existing_hard_link_aliases() {
    let temp = tempfile::tempdir().unwrap();
    let protected = temp.path().join("protected.json");
    let candidate = temp.path().join("candidate.json");
    std::fs::write(&protected, b"protected").unwrap();
    std::fs::hard_link(&protected, &candidate).unwrap();

    let error = crate::output::guarded_write_many(&candidate, std::iter::once(protected.as_path()))
        .unwrap_err();
    assert!(error.to_string().contains("OUTPUT_OVERWRITES_INPUT"));
}

#[test]
fn future_case_and_unicode_aliases_follow_the_actual_filesystem_policy() {
    let temp = tempfile::tempdir().unwrap();
    for (authored, alternate) in [
        ("VEAC-Protected.JSON", "veac-protected.json"),
        ("Caf\u{e9}.json", "Cafe\u{301}.json"),
    ] {
        let aliases = aliases_on_disk(temp.path(), authored, alternate);
        let protected = temp.path().join(authored);
        let candidate = temp.path().join(alternate);
        let result =
            crate::output::guarded_write_many(&candidate, std::iter::once(protected.as_path()));
        assert_eq!(result.is_err(), aliases, "{authored:?} vs {alternate:?}");
    }
}

fn aliases_on_disk(parent: &Path, authored: &str, alternate: &str) -> bool {
    let authored = parent.join(authored);
    let alternate = parent.join(alternate);
    std::fs::write(&authored, b"probe").unwrap();
    let aliases = std::fs::symlink_metadata(alternate).is_ok();
    std::fs::remove_file(authored).unwrap();
    aliases
}
