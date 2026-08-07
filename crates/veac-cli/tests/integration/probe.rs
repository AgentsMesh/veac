use std::io::Write;
use std::process::Command as ProcessCommand;

use veac_ir::{MaterialSource, MediaProbeSnapshot, StreamChoice};

use super::support::*;

#[test]
fn canonical_material_probe_uses_identity_uri_and_authored_stream_intent() {
    let temp = tempdir().unwrap();
    let media = temp.path().join("clip.mp4");
    make_av_fixture(&media);
    let project = compile_ir(&temp, MEDIA_SOURCE);
    set_global_video_intent(&project);

    let direct = run_probe(&media, None);
    assert_eq!(direct.selected_video_stream.unwrap().global_index, 0);
    assert_eq!(direct.selected_audio_stream.unwrap().global_index, 1);

    let material_id = read_project(&project).project.materials[0].id.to_string();
    let selected = run_probe(&project, Some(&material_id));
    assert_eq!(selected.schema_version, veac_ir::MEDIA_PROBE_SCHEMA_VERSION);
    assert_eq!(selected.streams.len(), 2);
    assert_eq!(selected.selected_video_stream.unwrap().global_index, 0);
    assert_eq!(selected.selected_video_stream.unwrap().type_index, 0);
    assert_eq!(selected.selected_audio_stream, None);
    assert_eq!(
        selected.observed_identity,
        veac_runtime::asset::sha256_identity(&media).unwrap()
    );
}

#[test]
fn canonical_material_probe_has_strict_missing_and_invalid_id_diagnostics() {
    let temp = tempdir().unwrap();
    let media = temp.path().join("clip.mp4");
    make_av_fixture(&media);
    let project = compile_ir(&temp, MEDIA_SOURCE);

    for (id, code) in [
        ("med_missing", "MATERIAL_NOT_FOUND"),
        ("bad id", "MATERIAL_ID_INVALID"),
    ] {
        let output = veac()
            .args([
                "--diagnostic-format",
                "json",
                "probe",
                project.to_str().unwrap(),
                "--material",
                id,
            ])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let diagnostics: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(diagnostics["diagnostics"][0]["code"], code);
    }
}

#[test]
fn canonical_material_probe_rejects_remote_and_tampered_sources() {
    let temp = tempdir().unwrap();
    let media = temp.path().join("clip.mp4");
    make_av_fixture(&media);
    let project = compile_ir(&temp, MEDIA_SOURCE);
    let mut envelope = read_project(&project);
    envelope.project.materials[0].source = MaterialSource::Remote {
        uri: "https://example.test/clip.mp4".to_owned(),
    };
    write_project(&project, &envelope);
    assert_failure(&project, "REMOTE_MATERIAL_UNRESOLVED");

    envelope.project.materials[0].source = MaterialSource::File {
        uri: "clip.mp4".to_owned(),
    };
    write_project(&project, &envelope);
    std::fs::OpenOptions::new()
        .append(true)
        .open(&media)
        .unwrap()
        .write_all(b"tampered")
        .unwrap();
    assert_failure(&project, "MATERIAL_IDENTITY_MISMATCH");
}

fn run_probe(input: &std::path::Path, material: Option<&str>) -> MediaProbeSnapshot {
    let mut command = veac();
    command.args(["probe", input.to_str().unwrap()]);
    if let Some(material) = material {
        command.args(["--material", material]);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn assert_failure(project: &std::path::Path, code: &str) {
    let material_id = read_project(project).project.materials[0].id.to_string();
    veac()
        .args([
            "probe",
            project.to_str().unwrap(),
            "--material",
            &material_id,
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(code));
}

fn read_project(path: &std::path::Path) -> veac_ir::ProjectEnvelope {
    veac_ir::decode_canonical_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn write_project(path: &std::path::Path, project: &veac_ir::ProjectEnvelope) {
    std::fs::write(path, veac_ir::canonical_json(project).unwrap()).unwrap();
}

fn set_global_video_intent(path: &std::path::Path) {
    let mut project = read_project(path);
    project.project.materials[0].stream_intent.video =
        StreamChoice::GlobalIndex { global_index: 0 };
    write_project(path, &project);
}

fn make_av_fixture(path: &std::path::Path) {
    let status = ProcessCommand::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=32x24:d=0.4:r=10",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=0.4",
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-pix_fmt",
            "yuv420p",
            "-shortest",
        ])
        .arg(path)
        .status()
        .expect("FFmpeg is required for CLI probe E2E tests");
    assert!(status.success());
}
