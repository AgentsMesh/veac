use super::support::*;

#[test]
fn canonical_generated_project_renders_through_real_ffmpeg() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, GENERATED_SOURCE);
    let output = temp.path().join("render.mp4");
    veac()
        .args(["render", project.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rendered:"));
    assert_eq!(video_dimensions(&output), "32x24");
}

#[test]
fn local_media_is_probed_bound_and_rendered_without_rewriting_ir() {
    let temp = tempdir().unwrap();
    make_video(&temp.path().join("clip.mp4"));
    let project = compile_ir(&temp, MEDIA_SOURCE);
    let before = std::fs::read(&project).unwrap();
    let plan = veac()
        .args(["plan", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        plan.status.success(),
        "{}",
        String::from_utf8_lossy(&plan.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&plan.stdout).unwrap();
    assert_eq!(value["inputs"][0]["canonical_uri"], "clip.mp4");
    assert_eq!(before, std::fs::read(&project).unwrap());

    let directory = temp.path().join("deliveries");
    std::fs::create_dir(&directory).unwrap();
    let output = directory.join("media-render.mp4");
    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--destination",
            directory.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(video_dimensions(&output), "32x24");
    assert_eq!(before, std::fs::read(&project).unwrap());
}

#[test]
fn render_requires_a_safe_existing_destination_directory() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, GENERATED_SOURCE);
    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--destination",
            project.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("OUTPUT_DIRECTORY_REQUIRED"));
    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--destination",
            temp.path().join("missing").to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("OUTPUT_DIRECTORY_REQUIRED"));
    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--destination",
            temp.path().join("wrong.webm").to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("OUTPUT_DIRECTORY_REQUIRED"));

    let store = temp.path().join(".veac-artifacts");
    std::fs::create_dir(&store).unwrap();
    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--destination",
            store.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("OUTPUT_RESERVED_DIRECTORY"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let directory = temp.path().join("real-directory");
        let link = temp.path().join("directory-link");
        std::fs::create_dir(&directory).unwrap();
        symlink(&directory, &link).unwrap();
        veac()
            .args([
                "render",
                project.to_str().unwrap(),
                "--destination",
                link.to_str().unwrap(),
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("OUTPUT_DIRECTORY_REQUIRED"));
    }
}

#[test]
fn probe_emits_normalized_versioned_json() {
    let temp = tempdir().unwrap();
    let media = temp.path().join("clip.mp4");
    make_video(&media);
    let output = veac()
        .args(["probe", media.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], veac_ir::MEDIA_PROBE_SCHEMA_VERSION);
    assert_eq!(value["selection_policy"], "veac.default-stream.v1");
    assert_eq!(value["observed_identity"]["algorithm"], "sha256");
    assert_eq!(
        value["observed_identity"]["digest"].as_str().unwrap().len(),
        64
    );
    assert!(value["engine"]
        .as_str()
        .unwrap()
        .starts_with("ffprobe version "));
    assert_eq!(value["selected_video_stream"]["global_index"], 0);
}

#[test]
fn probe_rejects_indirect_media_manifests() {
    let temp = tempdir().unwrap();
    let manifests = [
        (
            "playlist.m3u8",
            "#EXTM3U\n#EXT-X-TARGETDURATION:1\n#EXTINF:1,\nchild.ts\n#EXT-X-ENDLIST\n",
        ),
        ("files.ffconcat", "ffconcat version 1.0\nfile child.wav\n"),
    ];
    for (name, contents) in manifests {
        let path = temp.path().join(name);
        std::fs::write(&path, contents).unwrap();
        veac()
            .args(["probe", path.to_str().unwrap()])
            .assert()
            .failure()
            .stderr(predicate::str::contains("PROBE_FAILED"));
    }
}
