use std::path::Path;

use veac_codegen::emitter::{emit_all, BackendOutput, BackendPackagePaths, BackendProduct};
use veac_plan::canonical::*;

use super::delivery_extended_support::{ffmpeg, pair};
use super::support::{bindings, fixture, resolved, time};

#[test]
fn hls_rendition_ids_drive_stable_playlist_and_segment_arguments() {
    let mut plan = resolved(&fixture());
    plan.output.deliverables = vec![package(
        None,
        vec![
            rendition("rnd_hd", 1280, 720, HlsH264Profile::High),
            rendition("rnd_mobile", 640, 360, HlsH264Profile::Main),
        ],
    )];
    let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
    let task = &bundle.tasks()[0];
    assert_eq!(task.product, BackendProduct::HlsVod);
    assert_eq!(
        task.output,
        BackendOutput::Package {
            root: "/tmp/stream".into(),
            entrypoint: "master.m3u8".into(),
            paths: BackendPackagePaths {
                playlist_pattern: "rendition-%v.m3u8".into(),
                segment_pattern: "segment-%v-%06d.ts".into(),
            },
        }
    );
    let command = ffmpeg(&bundle, "dlv_hls");
    assert_eq!(command.output_path, Path::new("rendition-%v.m3u8"));
    for (name, value) in [
        ("-var_stream_map", "v:0,name:rnd_hd v:1,name:rnd_mobile"),
        ("-master_pl_name", "master.m3u8"),
        ("-hls_segment_filename", "segment-%v-%06d.ts"),
        ("-start_number", "0"),
        ("-profile:v:0", "high"),
        ("-profile:v:1", "main"),
        ("-level:v:0", "4"),
        ("-bf:v:0", "2"),
    ] {
        assert!(
            pair(&command.output_args, name, value),
            "missing {name}={value}"
        );
    }
    let graph = command.filter_graph.as_deref().unwrap();
    assert!(graph.contains("split=2"));
    assert!(graph.contains("scale=1280:720:flags=lanczos"));
    assert!(graph.contains("scale=640:360:flags=lanczos"));
}

#[test]
fn hls_aac_uses_its_declared_bus_source_and_encoding() {
    let mut project = fixture();
    enable_audio(&mut project);
    project.project.render_configs[0].deliverables = vec![package(
        Some(HlsAudio {
            source: AudioMixSource::Bus {
                bus_id: BusId::new("bus_dialogue").unwrap(),
            },
            encoding: HlsAudioEncoding::Aac(AacEncoding {
                bitrate_bps: 128_000,
                sample_rate_hz: 48_000,
                channel_layout: AudioChannelLayout::Stereo,
            }),
        }),
        vec![rendition("rnd_main", 1280, 720, HlsH264Profile::High)],
    )];
    let plan = resolved(&project);
    let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
    let command = ffmpeg(&bundle, "dlv_hls");
    for (name, value) in [
        ("-c:a:0", "aac"),
        ("-b:a:0", "128000"),
        ("-ar:a:0", "48000"),
        ("-ac:a:0", "2"),
        ("-var_stream_map", "v:0,a:0,name:rnd_main"),
    ] {
        assert!(pair(&command.output_args, name, value));
    }
    assert!(command
        .filter_graph
        .as_deref()
        .unwrap()
        .contains("[0:5]atrim="));
}

pub(super) fn package(audio: Option<HlsAudio>, renditions: Vec<HlsRendition>) -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_hls").unwrap(),
        target: DeliverableTarget::Package {
            name: "stream".into(),
        },
        kind: DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(HlsPackage {
            segment_duration: time(600),
            audio,
            renditions,
        })),
    }
}

pub(super) fn rendition(
    id: &str,
    width: u32,
    height: u32,
    profile: HlsH264Profile,
) -> HlsRendition {
    HlsRendition {
        id: HlsRenditionId::new(id).unwrap(),
        raster: HlsRenditionRaster { width, height },
        encoding: HlsVideoEncoding::H264(HlsH264Encoding {
            rate_control: HlsCappedBitrate {
                target_bps: 3_000_000,
                max_bps: 3_210_000,
                buffer_size_bits: 6_000_000,
            },
            profile: Some(profile),
            level: Some("4".into()),
            color_space: Some(ColorSpace {
                primaries: ColorPrimaries::Bt709,
                transfer: ColorTransfer::Bt709,
                matrix: ColorMatrix::Bt709,
                range: ColorRange::Limited,
            }),
            b_frames: Some(2),
        }),
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
