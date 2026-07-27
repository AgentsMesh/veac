use std::process::Command;

use tempfile::tempdir;
use veac_artifact::ArtifactStore;
use veac_codegen::emitter::BackendAction;

use super::support::*;

#[test]
fn vp9_and_av1_lossless_outputs_disable_the_bitrate_constraint() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.push(track(
        "trk_picture",
        TrackKind::Video,
        0,
        vec![solid_clip("itm_picture", color(20, 40, 80), 0, 1_000)],
    ));
    canonical.project.render_configs[0].deliverables = vec![
        video(
            "dlv_av1",
            "lossless.mkv",
            OutputFormat::Mkv,
            VideoCodec::Av1,
        ),
        video(
            "dlv_vp9",
            "lossless.webm",
            OutputFormat::Webm,
            VideoCodec::Vp9,
        ),
    ];
    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    for id in ["dlv_av1", "dlv_vp9"] {
        assert!(args(&delivery, id)
            .windows(2)
            .any(|pair| pair == ["-b:v", "0"]));
    }
    delivery.execute(&ArtifactStore::new(temp.path().join("store")));
    assert_eq!(codec(delivery.path("dlv_av1")), "av1");
    assert_eq!(codec(delivery.path("dlv_vp9")), "vp9");
}

fn video(id: &str, file: &str, container: OutputFormat, codec: VideoCodec) -> Deliverable {
    let settings = VideoDeliverable {
        container,
        video: VideoOutput {
            codec,
            rate_control: VideoRateControl::Lossless,
            ..VideoOutput::default()
        },
        audio: None,
        ..VideoDeliverable::default()
    };
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        file_name: file.to_owned(),
        kind: DeliverableKind::Video(settings),
    }
}

fn args<'a>(delivery: &'a PreparedDelivery, id: &str) -> &'a [String] {
    let task = delivery
        .bundle
        .tasks()
        .iter()
        .find(|task| task.deliverable_id.as_str() == id)
        .unwrap();
    let BackendAction::Ffmpeg(command) = &task.action else {
        panic!()
    };
    &command.output_args
}

fn codec(path: &Path) -> String {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name",
            "-of",
            "default=nw=1:nk=1",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}
