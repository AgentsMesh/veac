use super::*;

#[test]
fn accepts_one_exact_typed_layout() {
    let paths = paths("playlist-%v.m3u8", "segment-%v-%06d.ts");
    let command = command("master.m3u8", &paths);
    assert_eq!(
        validate_layout(&command, Path::new("master.m3u8"), &paths).unwrap(),
        2
    );
}

#[test]
fn rejects_an_output_path_mismatch() {
    let paths = paths("playlist-%v.m3u8", "segment-%v-%06d.ts");
    let mut command = command("master.m3u8", &paths);
    command.output_path = "other.m3u8".into();
    assert_error(&command, Path::new("master.m3u8"), &paths, "does not match");
}

#[test]
fn rejects_missing_and_duplicate_typed_options() {
    let paths = paths("playlist-%v.m3u8", "segment-%v-%06d.ts");
    for option in ["-master_pl_name", "-hls_segment_filename"] {
        let mut missing = command("master.m3u8", &paths);
        let index = missing
            .output_args
            .iter()
            .position(|value| value == option)
            .unwrap();
        missing.output_args.drain(index..=index + 1);
        assert_error(
            &missing,
            Path::new("master.m3u8"),
            &paths,
            "typed package layout",
        );

        let mut duplicate = command("master.m3u8", &paths);
        let value = duplicate.output_args[index + 1].clone();
        duplicate.output_args.extend([option.into(), value]);
        assert_error(
            &duplicate,
            Path::new("master.m3u8"),
            &paths,
            "typed package layout",
        );
    }
}

#[test]
fn rejects_each_unsafe_package_leaf() {
    let cases = [
        (
            "nested/master.m3u8",
            "playlist.m3u8",
            "segment.ts",
            "entrypoint",
        ),
        (
            "master.m3u8",
            "nested/playlist.m3u8",
            "segment.ts",
            "playlist pattern",
        ),
        (
            "master.m3u8",
            "playlist.m3u8",
            "../segment.ts",
            "segment pattern",
        ),
    ];
    for (entrypoint, playlist, segment, expected) in cases {
        let paths = paths(playlist, segment);
        let command = command(entrypoint, &paths);
        assert_error(&command, Path::new(entrypoint), &paths, expected);
    }
}

fn command(entrypoint: &str, paths: &BackendPackagePaths) -> BackendCommand {
    BackendCommand {
        preparations: Vec::new(),
        inputs: Vec::new(),
        filter_graph: None,
        filter_contract: None,
        maps: Vec::new(),
        output_args: vec![
            "-master_pl_name".into(),
            entrypoint.into(),
            "-hls_segment_filename".into(),
            output::path_string(&paths.segment_pattern),
        ],
        output_path: paths.playlist_pattern.clone(),
    }
}

fn paths(playlist: &str, segment: &str) -> BackendPackagePaths {
    BackendPackagePaths {
        playlist_pattern: playlist.into(),
        segment_pattern: segment.into(),
    }
}

fn assert_error(
    command: &BackendCommand,
    entrypoint: &Path,
    paths: &BackendPackagePaths,
    expected: &str,
) {
    let error = validate_layout(command, entrypoint, paths).unwrap_err();
    assert!(error.message.contains(expected), "{error}");
}
