use super::*;

fn time_base() -> Rational {
    Rational::new(1, 600).unwrap()
}

#[test]
fn complete_pts_and_durations_prove_cadence_and_rate() {
    let actual = classify(
        br#"{"packets":[{"pts":0,"duration":5},{"pts":15,"duration":5},{"pts":5,"duration":5},{"pts":10,"duration":5}]}"#,
        time_base(),
        None,
    );
    assert_eq!(actual.cadence, VideoCadence::Constant);
    assert_eq!(actual.frame_rate, Some(Rational::new(120, 1).unwrap()));

    assert_eq!(
        classify(
            br#"{"packets":[{"pts":0,"duration":5},{"pts":5,"duration":5},{"pts":20,"duration":5}]}"#,
            time_base(),
            None,
        )
        .cadence,
        VideoCadence::Variable
    );
    assert_eq!(
        classify(
            br#"{"packets":[{"pts":0,"duration":5},{"pts":5,"duration":5},{"pts":10,"duration":4}]}"#,
            time_base(),
            None,
        )
        .cadence,
        VideoCadence::Variable
    );
}

#[test]
fn missing_malformed_duplicate_or_excessive_evidence_fails_closed() {
    for value in [
        b"".as_slice(),
        br#"{"packets":[{"pts":0,"duration":5}]}"#,
        br#"{"packets":[{"pts":0},{"pts":5,"duration":5}]}"#,
        br#"{"packets":[{"pts":"N/A","duration":5}]}"#,
        &[0xff, b'\n'],
    ] {
        assert_eq!(
            classify(value, time_base(), None).cadence,
            VideoCadence::Unknown
        );
    }
    assert_eq!(
        classify(
            br#"{"packets":[{"pts":0,"duration":5},{"pts":0,"duration":5}]}"#,
            time_base(),
            None,
        )
        .cadence,
        VideoCadence::Variable
    );
    assert_eq!(
        classify(
            br#"{"packets":[{"pts":0,"duration":1},{"pts":1,"duration":1}]}"#,
            Rational::new(1, 90_000).unwrap(),
            None,
        )
        .cadence,
        VideoCadence::Unknown
    );
}

#[test]
fn candidate_allows_one_tick_timebase_quantization() {
    let evidence = classify(
        br#"{"packets":[{"pts":0,"duration":33},{"pts":33,"duration":33},{"pts":67,"duration":33},{"pts":100,"duration":33},{"pts":133,"duration":33},{"pts":167,"duration":33}]}"#,
        Rational::new(1, 1_000).unwrap(),
        Some(Rational::new(30_000, 1_001).unwrap()),
    );
    assert_eq!(evidence.cadence, VideoCadence::Constant);
    assert_eq!(
        evidence.frame_rate,
        Some(Rational::new(30_000, 1_001).unwrap())
    );
}

#[test]
fn candidate_does_not_hide_variable_packet_cadence() {
    let evidence = classify(
        br#"{"packets":[{"pts":0,"duration":50},{"pts":50,"duration":50},{"pts":100,"duration":50},{"pts":250,"duration":50}]}"#,
        Rational::new(1, 1_000).unwrap(),
        Some(Rational::new(20, 1).unwrap()),
    );
    assert_eq!(evidence.cadence, VideoCadence::Variable);

    for packets in [
        br#"{"packets":[{"pts":0,"duration":33},{"pts":0,"duration":33}]}"#.as_slice(),
        br#"{"packets":[{"pts":0,"duration":0},{"pts":33,"duration":33}]}"#,
    ] {
        assert_eq!(
            classify(
                packets,
                Rational::new(1, 1_000).unwrap(),
                Some(Rational::new(30_000, 1_001).unwrap()),
            )
            .cadence,
            VideoCadence::Variable
        );
    }
}

#[test]
fn arithmetic_boundaries_fail_closed_without_overflowing() {
    let extreme_pts = format!(
        "{{\"packets\":[{{\"pts\":{},\"duration\":1}},{{\"pts\":{},\"duration\":1}}]}}",
        i64::MIN,
        i64::MAX
    );
    assert_eq!(
        classify(extreme_pts.as_bytes(), time_base(), None).cadence,
        VideoCadence::Unknown
    );
    let large_delta = format!(
        "{{\"packets\":[{{\"pts\":0,\"duration\":{0}}},{{\"pts\":{0},\"duration\":{0}}}]}}",
        i64::MAX
    );
    assert_eq!(
        classify(
            large_delta.as_bytes(),
            Rational::new(veac_ir::MAX_SAFE_INTEGER as i64, 1).unwrap(),
            None,
        )
        .cadence,
        VideoCadence::Unknown
    );
    let packets = br#"{"packets":[{"pts":0,"duration":2},{"pts":2,"duration":2}]}"#;
    assert_eq!(
        classify(packets, Rational::new(0, 1).unwrap(), None).cadence,
        VideoCadence::Unknown
    );
    assert_eq!(
        classify(
            packets,
            Rational::new(-1, 1).unwrap(),
            Some(Rational::new(1, 1).unwrap()),
        )
        .cadence,
        VideoCadence::Unknown
    );
}
