use veac_ir::{HashAlgorithm, MediaIdentity, ProbedStreamType, StreamDisposition, StreamSelection};

use super::*;

#[test]
fn nonzero_stream_window_is_used_as_an_absolute_interval() {
    let stream = stream(Some(time(5, 1)), Some(time(10, 1)));
    let probe = probe(stream.clone(), Some(time(99, 1)));
    assert!(require_range(&probe, &stream, time(5, 1), time(15, 1)).is_ok());
    assert!(require_range(&probe, &stream, time(4, 1), time(6, 1)).is_err());
    assert!(require_range(&probe, &stream, time(14, 1), time(15, 1)).is_ok());
    assert!(require_point(&probe, &stream, time(15, 1)).is_err());
}

#[test]
fn container_fallback_never_invents_a_nonzero_stream_end() {
    let selected = stream(Some(time(5, 1)), None);
    let snapshot = probe(selected.clone(), Some(time(15, 1)));
    assert!(require_range(&snapshot, &selected, time(5, 1), time(10, 1)).is_err());

    let zero = stream(None, None);
    let snapshot = probe(zero.clone(), Some(time(15, 1)));
    assert!(require_range(&snapshot, &zero, time(0, 1), time(15, 1)).is_ok());
}

#[test]
fn mixed_timescales_and_overflow_fail_closed() {
    let selected = stream(Some(time(10, 2)), Some(time(30, 3)));
    let snapshot = probe(selected.clone(), None);
    assert!(require_range(&snapshot, &selected, time(25, 5), time(45, 3)).is_ok());

    let invalid = RationalTime {
        value: i64::MAX,
        timescale: 1,
    };
    let huge = stream(Some(invalid), Some(invalid));
    let snapshot = probe(huge.clone(), None);
    assert!(require_range(&snapshot, &huge, invalid, invalid).is_err());
}

fn time(value: i64, timescale: u32) -> RationalTime {
    RationalTime::new(value, timescale).unwrap()
}

fn stream(start_time: Option<RationalTime>, duration: Option<RationalTime>) -> ProbedStream {
    ProbedStream {
        global_index: 0,
        type_index: 0,
        media_type: ProbedStreamType::Video,
        codec: "h264".into(),
        time_base: None,
        start_time,
        duration,
        disposition: StreamDisposition {
            default: true,
            attached_picture: false,
            timed_thumbnail: false,
        },
        video: None,
        audio: None,
    }
}

fn probe(stream: ProbedStream, container_duration: Option<RationalTime>) -> MediaProbeSnapshot {
    MediaProbeSnapshot {
        schema_version: veac_ir::MEDIA_PROBE_SCHEMA_VERSION,
        engine: "test".into(),
        selection_policy: "test".into(),
        container_format: "test".into(),
        container_brand: None,
        observed_identity: MediaIdentity {
            algorithm: HashAlgorithm::Sha256,
            digest: "ab".repeat(32),
        },
        container_duration,
        streams: vec![stream],
        selected_video_stream: Some(StreamSelection {
            global_index: 0,
            type_index: 0,
        }),
        selected_audio_stream: None,
    }
}
