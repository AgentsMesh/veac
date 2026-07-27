use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;
use veac_artifact::ArtifactStore;

use super::support::*;

#[test]
fn high_precision_image_sequences_and_pcm_stems_are_real_and_cached() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.push(track(
        "trk_source",
        TrackKind::Video,
        0,
        vec![solid_clip("itm_source", color(220, 40, 90), 0, 1_000)],
    ));
    canonical.project.render_configs[0].deliverables = vec![
        image("dlv_exr", "exr-%d.exr", ImageFormat::Exr),
        stem("dlv_pcm24", "pcm24.wav", AudioCodec::PcmS24Le),
        stem("dlv_pcm32", "pcm32.wav", AudioCodec::PcmS32Le),
        image("dlv_tiff", "tiff-%d.tiff", ImageFormat::Tiff),
    ];
    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    let store = ArtifactStore::new(temp.path().join("store"));
    let first = delivery.execute(&store);
    assert_eq!(first.tasks.len(), 4);
    assert!(first.tasks.iter().all(|task| !task.cache_hit));
    for (id, codec, pixel) in [
        ("dlv_exr", "exr", "gbrapf16le"),
        ("dlv_tiff", "tiff", "rgba64le"),
    ] {
        let task = first
            .tasks
            .iter()
            .find(|task| task.deliverable_id.as_str() == id)
            .unwrap();
        assert_eq!(task.outputs.len(), FPS as usize);
        let stream = probe(&task.outputs[0], "v:0");
        assert_eq!(stream["codec_name"], codec);
        assert_eq!(stream["pix_fmt"], pixel);
    }
    for (id, codec, bits) in [
        ("dlv_pcm24", "pcm_s24le", "24"),
        ("dlv_pcm32", "pcm_s32le", "32"),
    ] {
        let stream = probe(delivery.path(id), "a:0");
        assert_eq!(stream["codec_name"], codec);
        assert_eq!(stream["sample_fmt"], "s32");
        assert_eq!(stream["sample_rate"], "96000");
        assert_eq!(stream["bits_per_raw_sample"], bits);
    }
    assert!(delivery
        .execute(&store)
        .tasks
        .iter()
        .all(|task| task.cache_hit));
}

fn image(id: &str, file: &str, format: ImageFormat) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        file_name: file.to_owned(),
        kind: DeliverableKind::ImageSequence(ImageSequenceOutput {
            format,
            start_number: 1,
        }),
    }
}

fn stem(id: &str, file: &str, codec: AudioCodec) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        file_name: file.to_owned(),
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec,
                sample_rate: 96_000,
                channels: 2,
            },
            source: AudioStemSource::Master,
        }),
    }
}

fn probe(path: &Path, stream: &str) -> Value {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            stream,
            "-show_entries",
            "stream=codec_name,pix_fmt,sample_fmt,sample_rate,bits_per_raw_sample",
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
