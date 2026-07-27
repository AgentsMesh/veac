use std::panic::catch_unwind;

use crate::test_support::{multicam_project, time};

use super::*;

#[test]
fn zero_timebase_source_curve_returns_typed_diagnostics_without_panicking() {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].clips[0].source_mapping =
        Some(source_curve(vec![SourceTimeSegment {
            record_duration: time(600),
            source_start: time(0),
            source_end: time(600),
            interpolation: SourceTimeInterpolation::Linear,
        }]));
    project.project.timebase = 0;

    let codes = caught_validation_codes(&project);
    for code in ["TIMEBASE", "SOURCE_TIME_MAP", "SOURCE_TIME"] {
        assert_code(&codes, code);
    }

    project.project.sequences[0].tracks[0].clips[0].source_mapping = Some(source_curve(vec![]));
    let codes = caught_validation_codes(&project);
    assert_code(&codes, "TIMEBASE");
    assert_code(&codes, "SOURCE_TIME_MAP_EMPTY");
}

#[test]
fn zero_timebase_multicam_clip_returns_typed_diagnostics_without_panicking() {
    let mut project = multicam_project();
    project.project.timebase = 0;

    let codes = caught_validation_codes(&project);
    for code in [
        "TIMEBASE",
        "MULTICAM_SOURCE_OFFSET",
        "MULTICAM_SWITCH_RANGE",
    ] {
        assert_code(&codes, code);
    }

    let ClipSource::Multicam { switches, .. } =
        &mut project.project.sequences[0].tracks[0].clips[0].source
    else {
        unreachable!()
    };
    switches.clear();
    let codes = caught_validation_codes(&project);
    assert_code(&codes, "TIMEBASE");
    assert_code(&codes, "MULTICAM_SWITCH_PARTITION");
}

fn caught_validation_codes(project: &ProjectEnvelope) -> Vec<String> {
    match catch_unwind(|| validate(project)) {
        Ok(Err(errors)) => errors
            .into_diagnostics()
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect(),
        Ok(Ok(())) => panic!("zero project timebase must return validation diagnostics"),
        Err(_) => panic!("validation must not panic for an untrusted canonical DTO"),
    }
}

fn source_curve(segments: Vec<SourceTimeSegment>) -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Curve { segments },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::Strict,
    }
}
