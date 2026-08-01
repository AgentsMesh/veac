use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;
use veac_artifact::ArtifactStore;
use veac_ir::StreamChoice;

use super::support::*;

#[test]
fn master_track_and_bus_stems_have_real_codec_routing_and_energy() {
    let temp = tempdir().unwrap();
    let dialogue = tone_fixture(temp.path(), "dialogue", 440);
    let music = tone_fixture(temp.path(), "music", 880);
    let mut canonical = project(false);
    canonical.project.render_configs[0].raster = None;
    canonical.project.materials.extend([
        material(
            "med_dialogue",
            MaterialKind::Audio,
            StreamChoice::Disabled,
            StreamChoice::Auto,
        ),
        material(
            "med_music",
            MaterialKind::Audio,
            StreamChoice::Disabled,
            StreamChoice::Auto,
        ),
    ]);
    let mut dialogue_clip = media_clip("itm_dialogue", "med_dialogue", 0, 1_000);
    dialogue_clip.audio = Some(audio_properties(0.35));
    let mut music_clip = media_clip("itm_music", "med_music", 0, 1_000);
    music_clip.audio = Some(audio_properties(0.35));
    let mut dialogue_track = track("trk_dialogue", TrackKind::Audio, 1, vec![dialogue_clip]);
    dialogue_track.routing = TrackRouting::AudioBus {
        bus_id: BusId::new("bus_dialogue").unwrap(),
    };
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_picture",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_picture", color(20, 40, 80), 0, 1_000)],
        ),
        dialogue_track,
        track("trk_music", TrackKind::Audio, 2, vec![music_clip]),
    ]);
    canonical.project.render_configs[0].deliverables = vec![
        stem(
            "dlv_bus",
            "dialogue.flac",
            AudioStemFormat::Flac,
            AudioMixSource::Bus {
                bus_id: BusId::new("bus_dialogue").unwrap(),
            },
        ),
        stem(
            "dlv_master",
            "master.wav",
            AudioStemFormat::Wav,
            AudioMixSource::Master,
        ),
        stem(
            "dlv_track",
            "track.wav",
            AudioStemFormat::Wav,
            AudioMixSource::Track {
                track_id: TrackId::new("trk_dialogue").unwrap(),
            },
        ),
    ];
    let assets = BTreeMap::from([
        ("med_dialogue".to_owned(), dialogue),
        ("med_music".to_owned(), music),
    ]);
    let delivery = prepare_delivery(canonical, &assets, temp.path());
    delivery.execute(&ArtifactStore::new(temp.path().join("store")));
    for (id, codec) in [
        ("dlv_bus", "flac"),
        ("dlv_master", "pcm_s16le"),
        ("dlv_track", "pcm_s16le"),
    ] {
        let stream = audio_stream(delivery.path(id));
        assert_eq!(stream["codec_name"], codec);
        assert_eq!(stream["sample_rate"], "48000");
        assert_eq!(stream["channels"], 1);
        assert!(rms_db(&audio_samples(delivery.path(id), 0.1, 0.5)) > -40.0);
    }
    let bus = audio_samples(delivery.path("dlv_bus"), 0.1, 0.5);
    let track = audio_samples(delivery.path("dlv_track"), 0.1, 0.5);
    let master = audio_samples(delivery.path("dlv_master"), 0.1, 0.5);
    assert!(tone_power(&bus, 440.0) > tone_power(&bus, 880.0) * 8.0);
    assert!(tone_power(&track, 440.0) > tone_power(&track, 880.0) * 8.0);
    assert!(tone_power(&master, 440.0) > 0.005);
    assert!(tone_power(&master, 880.0) > 0.005);
}

fn stem(id: &str, file: &str, format: AudioStemFormat, source: AudioMixSource) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target: DeliverableTarget::File {
            name: file.to_owned(),
        },
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format,
            audio: AudioOutput {
                codec: match format {
                    AudioStemFormat::Wav => AudioCodec::PcmS16Le,
                    AudioStemFormat::Flac => AudioCodec::Flac,
                },
                sample_rate: 48_000,
                channels: 1,
            },
            source,
        }),
    }
}

fn audio_stream(path: &Path) -> Value {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=codec_name,sample_rate,channels",
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
