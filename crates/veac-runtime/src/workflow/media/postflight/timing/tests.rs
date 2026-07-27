use super::*;

#[test]
fn video_duration_is_bounded_by_one_frame_and_rounding() {
    let rate = veac_ir::Rational::new(25, 1).unwrap();
    assert!(video(time(999_000), time(1_000_000), rate));
    assert!(!video(time(998_999), time(1_000_000), rate));
    assert!(video(time(1_041_000), time(1_000_000), rate));
    assert!(!video(time(1_041_001), time(1_000_000), rate));
}

#[test]
fn audio_duration_is_bounded_by_one_sample_and_rounding() {
    assert!(audio(time(999_000), time(1_000_000), 48_000));
    assert!(audio(time(1_001_020), time(1_000_000), 48_000));
    assert!(!audio(time(1_001_021), time(1_000_000), 48_000));
    let invalid = RationalTime {
        value: i64::MAX,
        timescale: 1_000_000,
    };
    assert!(!audio(invalid, time(1_000_000), 48_000));
}

#[test]
fn missing_origin_requires_explicit_container_allowance() {
    let mut stream = stream(None);
    assert!(super::super::snapshot::zero_origin(&stream, true).is_ok());
    assert!(super::super::snapshot::zero_origin(&stream, false).is_err());
    stream.start_time = Some(time(1));
    assert!(super::super::snapshot::zero_origin(&stream, true).is_err());
    stream.start_time = Some(time(0));
    assert!(super::super::snapshot::zero_origin(&stream, false).is_ok());
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 1_000_000).unwrap()
}

fn stream(start_time: Option<RationalTime>) -> veac_ir::ProbedStream {
    veac_ir::ProbedStream {
        global_index: 0,
        type_index: 0,
        media_type: veac_ir::ProbedStreamType::Video,
        codec: "png".into(),
        time_base: None,
        start_time,
        duration: None,
        disposition: veac_ir::StreamDisposition {
            default: true,
            attached_picture: false,
            timed_thumbnail: false,
        },
        video: None,
        audio: None,
    }
}
