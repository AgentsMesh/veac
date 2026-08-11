use super::super::support::*;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command as ProcessCommand;

const PROJECT: &str =
    include_str!("../../../../veac-project/tests/fixtures/authored_derivation_matrix.veac");

#[test]
fn project_build_executes_and_caches_every_remaining_derivation() {
    let temp = tempdir().unwrap();
    let entry = prepare(&temp, PROJECT);
    let receipt = temp.path().join("build/receipt.json");

    let first = invoke(&entry, &receipt);
    assert!(
        first.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&first.stderr),
        String::from_utf8_lossy(&std::fs::read(&receipt).unwrap())
    );
    assert_statuses(&receipt, "executed", 5);
    assert_outputs(temp.path());

    let second = invoke(&entry, &receipt);
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_statuses(&receipt, "cache_hit", 5);
    assert_outputs(temp.path());
}

#[test]
fn project_build_rejects_a_derivation_output_contract_mismatch() {
    let temp = tempdir().unwrap();
    let invalid = PROJECT.replacen("MediaType.Audio", "MediaType.Video", 1);
    let entry = prepare(&temp, &invalid);
    veac()
        .args(["project", "build"])
        .arg(entry)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "media derivation requires exactly one Audio media output",
        ));
    assert!(!temp.path().join("dist/proxy-audio.wav").exists());
    assert!(!temp.path().join(".cache/veac").exists());
}

fn prepare(temp: &TempDir, project: &str) -> std::path::PathBuf {
    let entry = temp.path().join("project.veac");
    std::fs::create_dir(temp.path().join("sources")).unwrap();
    let materials = temp.path().join("materials");
    std::fs::create_dir(&materials).unwrap();
    std::fs::write(&entry, project).unwrap();
    make_av(&materials.join("source.mp4"));
    entry
}

fn invoke(entry: &Path, receipt: &Path) -> std::process::Output {
    veac()
        .args(["project", "build"])
        .arg(entry)
        .arg("--receipt")
        .arg(receipt)
        .output()
        .unwrap()
}

fn assert_statuses(receipt: &Path, expected: &str, count: usize) {
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(receipt).unwrap()).unwrap();
    let nodes = value["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), count);
    assert!(nodes.iter().all(|node| node["status"] == expected));
    for field in ["action_digest", "cache_key"] {
        let digests = nodes
            .iter()
            .map(|node| node[field]["value"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(digests.len(), count);
    }
}

fn assert_outputs(root: &Path) {
    let dist = root.join("dist");
    assert_eq!(
        stream(
            &dist.join("proxy-audio.wav"),
            "a:0",
            "codec_name,sample_rate,channels"
        ),
        "pcm_s16le,16000,1"
    );
    assert_eq!(video_dimensions(&dist.join("thumbnail.png")), "20x14");
    assert_eq!(video_dimensions(&dist.join("waveform.png")), "64x24");
    assert_eq!(video_dimensions(&dist.join("optical-flow.mp4")), "32x32");
    assert_eq!(video_dimensions(&dist.join("source-segment.mp4")), "40x30");
    assert_eq!(
        stream(
            &dist.join("source-segment.mp4"),
            "a:0",
            "sample_rate,channels"
        ),
        "22050,1"
    );
}

fn stream(path: &Path, selection: &str, fields: &str) -> String {
    let output = ProcessCommand::new("ffprobe")
        .args(["-v", "error", "-select_streams", selection, "-show_entries"])
        .arg(format!("stream={fields}"))
        .args(["-of", "csv=p=0"])
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn make_av(path: &Path) {
    let status = ProcessCommand::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
        ])
        .arg("testsrc2=size=64x48:rate=12:duration=1")
        .args([
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=1",
        ])
        .args([
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-ar",
            "48000",
            "-ac",
            "2",
            "-shortest",
        ])
        .arg(path)
        .status()
        .expect("FFmpeg is required for project derivation E2E tests");
    assert!(status.success());
}
