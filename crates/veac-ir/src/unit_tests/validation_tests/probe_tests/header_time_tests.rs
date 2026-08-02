use super::*;

#[test]
fn every_probe_header_field_is_required_independently() {
    for mutate in [
        |probe: &mut MediaProbeSnapshot| probe.schema_version = 1,
        |probe: &mut MediaProbeSnapshot| probe.engine = "  ".to_owned(),
        |probe: &mut MediaProbeSnapshot| probe.selection_policy = "".to_owned(),
        |probe: &mut MediaProbeSnapshot| probe.container_format = "mov,MP4".to_owned(),
        |probe: &mut MediaProbeSnapshot| probe.container_brand = Some("x".repeat(65)),
        |probe: &mut MediaProbeSnapshot| probe.container_brand = Some("bad\nbrand".to_owned()),
        |probe: &mut MediaProbeSnapshot| probe.observed_identity.algorithm = HashAlgorithm::Blake3,
        |probe: &mut MediaProbeSnapshot| probe.observed_identity.digest = "A".repeat(64),
    ] {
        let mut project = sample_project();
        mutate(probe_mut(&mut project));
        assert_probe_code(&project, "PROBE_HEADER");
    }
}

#[test]
fn probe_identity_may_be_unpinned_but_must_match_when_pinned() {
    let mut unpinned = sample_project();
    material_mut(&mut unpinned).identity = None;
    validate(&unpinned).unwrap();

    let mut mismatch = sample_project();
    probe_mut(&mut mismatch).observed_identity.digest = "b".repeat(64);
    assert_probe_code(&mismatch, "PROBE_IDENTITY");
}

#[test]
fn stream_times_allow_absence_and_reject_negative_zero_or_invalid_values() {
    let mut absent = sample_project();
    let stream = &mut probe_mut(&mut absent).streams[0];
    stream.start_time = None;
    stream.duration = None;
    validate(&absent).unwrap();

    let invalid = [
        (
            Some(RationalTime {
                value: -1,
                timescale: 600,
            }),
            None,
        ),
        (
            None,
            Some(RationalTime {
                value: 0,
                timescale: 600,
            }),
        ),
        (
            Some(RationalTime {
                value: 0,
                timescale: 0,
            }),
            None,
        ),
    ];
    for (start, duration) in invalid {
        let mut project = sample_project();
        let stream = &mut probe_mut(&mut project).streams[0];
        stream.start_time = start;
        stream.duration = duration;
        assert_probe_code(&project, "PROBE_STREAM_TIME");
    }
}
