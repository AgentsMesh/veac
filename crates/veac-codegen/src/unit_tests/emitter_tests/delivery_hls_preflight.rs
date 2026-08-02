use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;

use super::delivery_hls::{package, rendition};
use super::support::{bindings, fixture, resolved};

#[test]
fn hls_rendition_identity_and_raster_sets_are_closed() {
    let duplicate_id = vec![
        rendition("rnd_same", 1280, 720, HlsH264Profile::High),
        rendition("rnd_same", 640, 360, HlsH264Profile::Main),
    ];
    assert_invalid(package(None, duplicate_id));

    let unsorted = vec![
        rendition("rnd_z", 1280, 720, HlsH264Profile::High),
        rendition("rnd_a", 640, 360, HlsH264Profile::Main),
    ];
    assert_invalid(package(None, unsorted));

    let duplicate_raster = vec![
        rendition("rnd_a", 1280, 720, HlsH264Profile::High),
        rendition("rnd_b", 1280, 720, HlsH264Profile::Main),
    ];
    assert_invalid(package(None, duplicate_raster));
}

#[test]
fn hls_baseline_b_frames_and_invalid_aac_source_fail_closed() {
    let mut subsecond = package(
        None,
        vec![rendition("rnd_main", 1280, 720, HlsH264Profile::High)],
    );
    let DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(settings)) = &mut subsecond.kind
    else {
        unreachable!()
    };
    settings.segment_duration.value -= 1;
    assert_invalid(subsecond);

    assert_invalid(package(
        None,
        vec![rendition("rnd_main", 1280, 720, HlsH264Profile::Baseline)],
    ));

    let audio = HlsAudio {
        source: AudioMixSource::Track {
            track_id: TrackId::new("trk_missing").unwrap(),
        },
        encoding: HlsAudioEncoding::Aac(AacEncoding {
            bitrate_bps: 700_000,
            sample_rate_hz: 48_000,
            channel_layout: AudioChannelLayout::Stereo,
        }),
    };
    assert_invalid(package(
        Some(audio),
        vec![rendition("rnd_main", 1280, 720, HlsH264Profile::High)],
    ));
}

fn assert_invalid(deliverable: Deliverable) {
    let mut plan = resolved(&fixture());
    plan.output.deliverables = vec![deliverable];
    let error = emit_all(&plan, &bindings(&plan)).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|value| value.code == "PLAN_HLS_PACKAGE_INVALID"));
}
