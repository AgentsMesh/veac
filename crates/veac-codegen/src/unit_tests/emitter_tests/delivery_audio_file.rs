use veac_codegen::emitter::{emit_all, BackendProduct};
use veac_plan::canonical::*;

use super::delivery_extended_support::{ffmpeg, file, pair};
use super::support::{bindings, fixture, resolved};

#[test]
fn mp3_master_track_and_bus_sources_emit_the_declared_contract() {
    for (suffix, source) in [
        ("master", AudioMixSource::Master),
        (
            "track",
            AudioMixSource::Track {
                track_id: TrackId::new("trk_video").unwrap(),
            },
        ),
        (
            "bus",
            AudioMixSource::Bus {
                bus_id: BusId::new("bus_dialogue").unwrap(),
            },
        ),
    ] {
        let mut project = fixture();
        enable_audio(&mut project);
        let id = format!("dlv_{suffix}");
        project.project.render_configs[0].raster = None;
        project.project.render_configs[0].deliverables = vec![mp3(&id, source)];
        let plan = resolved(&project);
        let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
        let task = &bundle.tasks()[0];
        assert_eq!(task.product, BackendProduct::AudioFile);
        let command = ffmpeg(&bundle, &id);
        let graph = command.filter_graph.as_deref().unwrap();
        assert!(
            graph.contains("[0:5]atrim="),
            "missing selected audio: {graph}"
        );
        for (name, value) in [
            ("-c:a", "libmp3lame"),
            ("-b:a", "192000"),
            ("-ar", "48000"),
            ("-ac", "2"),
            ("-f", "mp3"),
        ] {
            assert!(pair(&command.output_args, name, value));
        }
        assert!(command.output_args.contains(&"-vn".to_owned()));
    }
}

fn enable_audio(project: &mut ProjectEnvelope) {
    let track = &mut project.project.sequences[0].tracks[0];
    track.routing = TrackRouting::AudioBus {
        bus_id: BusId::new("bus_dialogue").unwrap(),
    };
    track.clips[0].audio = Some(AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    });
}

fn mp3(id: &str, source: AudioMixSource) -> Deliverable {
    file(
        id,
        &format!("{id}.mp3"),
        DeliverableKind::AudioFile(AudioFile {
            source,
            encoding: AudioFileEncoding::Mp3(Mp3Encoding {
                bitrate_bps: 192_000,
                sample_rate_hz: 48_000,
                channel_layout: AudioChannelLayout::Stereo,
            }),
        }),
    )
}
