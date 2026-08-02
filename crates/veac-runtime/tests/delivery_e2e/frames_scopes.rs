use std::collections::BTreeSet;
use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;
use veac_artifact::{ArtifactStore, ContentDigest};

use super::support::*;

#[test]
fn image_sequences_and_scopes_are_real_distinct_inspectable_outputs() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.push(track(
        "trk_source",
        TrackKind::Video,
        0,
        vec![solid_clip("itm_source", color(220, 40, 90), 0, 1_000)],
    ));
    canonical.project.render_configs[0].deliverables = vec![
        scope(
            "dlv_histogram",
            "histogram.jpg",
            VideoScope::Histogram,
            ImageFormat::Jpeg,
        ),
        sequence("dlv_jpeg", "jpeg-%d.jpg", ImageFormat::Jpeg, 100),
        sequence("dlv_png", "png-%d.png", ImageFormat::Png, 1),
        scope(
            "dlv_vectorscope",
            "vectorscope.png",
            VideoScope::Vectorscope,
            ImageFormat::Png,
        ),
        scope(
            "dlv_waveform",
            "waveform.png",
            VideoScope::Waveform,
            ImageFormat::Png,
        ),
    ];
    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    let store = ArtifactStore::new(temp.path().join("store"));
    let first = delivery.execute(&store);
    assert_eq!(first.tasks.len(), 5);
    for id in ["dlv_jpeg", "dlv_png"] {
        let task = first
            .tasks
            .iter()
            .find(|task| task.deliverable_id.as_str() == id)
            .unwrap();
        assert_eq!(task.outputs.len(), FPS as usize);
        let image = image_stream(&task.outputs[0]);
        assert_eq!(image["width"], WIDTH);
        assert_eq!(image["height"], HEIGHT);
        assert_eq!(
            image["codec_name"],
            if id == "dlv_png" { "png" } else { "mjpeg" }
        );
    }

    let mut scope_digests = BTreeSet::new();
    for id in ["dlv_histogram", "dlv_vectorscope", "dlv_waveform"] {
        let path = delivery.path(id);
        assert!(std::fs::metadata(path).unwrap().len() > 100);
        let image = image_stream(path);
        assert_eq!(image["width"], 64);
        assert_eq!(image["height"], 48);
        let rgb = rgb_image(path, 64, 48);
        assert!(rgb.iter().any(|value| *value != rgb[0]));
        scope_digests.insert(ContentDigest::sha256(rgb).value);
    }
    assert_eq!(
        scope_digests.len(),
        3,
        "scope images must be visually distinct"
    );
    let resumed = delivery.execute(&store);
    assert!(resumed.tasks.iter().all(|task| task.cache_hit));
}

#[test]
fn scope_at_the_last_timeline_tick_selects_the_containing_frame() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.push(track(
        "trk_tail_scope",
        TrackKind::Video,
        0,
        vec![solid_clip("itm_tail_scope", color(30, 180, 80), 0, 1_000)],
    ));
    let mut deliverable = scope(
        "dlv_tail_scope",
        "tail-scope.png",
        VideoScope::Waveform,
        ImageFormat::Png,
    );
    let DeliverableKind::Scope(settings) = &mut deliverable.kind else {
        unreachable!()
    };
    settings.at = time(999);
    canonical.project.render_configs[0].deliverables = vec![deliverable];
    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    let store = ArtifactStore::new(temp.path().join("store"));
    let result = delivery.execute(&store);

    assert_eq!(result.tasks.len(), 1);
    let output = delivery.path("dlv_tail_scope");
    assert!(std::fs::metadata(output).unwrap().len() > 100);
    assert_eq!(image_stream(output)["codec_name"], "png");
}

fn sequence(id: &str, file: &str, format: ImageFormat, start_number: u32) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target: DeliverableTarget::ImageSequence {
            pattern: file.to_owned(),
        },
        kind: DeliverableKind::ImageSequence(ImageSequenceOutput {
            format,
            start_number,
        }),
    }
}

fn scope(id: &str, file: &str, scope: VideoScope, format: ImageFormat) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target: DeliverableTarget::File {
            name: file.to_owned(),
        },
        kind: DeliverableKind::Scope(ScopeOutput {
            scope,
            at: time(500),
            width: 64,
            height: 48,
            format,
        }),
    }
}

fn image_stream(path: &Path) -> Value {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name,width,height,pix_fmt",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["streams"][0].clone()
}

fn rgb_image(path: &Path, width: u32, height: u32) -> Vec<u8> {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args(["-frames:v", "1", "-pix_fmt", "rgb24", "-f", "rawvideo", "-"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.len(), (width * height * 3) as usize);
    output.stdout
}
