use super::*;

#[test]
fn every_scope_kind_and_image_format_is_valid_inside_the_sequence() {
    for value in [
        scope("waveform.PNG", ImageFormat::Png, VideoScope::Waveform),
        scope(
            "vectorscope.JPG",
            ImageFormat::Jpeg,
            VideoScope::Vectorscope,
        ),
        scope("histogram.png", ImageFormat::Png, VideoScope::Histogram),
    ] {
        assert_valid(value);
    }
}

#[test]
fn scopes_reject_each_geometry_format_and_sequence_range_failure() {
    let mut width = scope("scope.png", ImageFormat::Png, VideoScope::Waveform);
    scope_settings(&mut width).width = 0;
    let mut height = scope("scope.png", ImageFormat::Png, VideoScope::Waveform);
    scope_settings(&mut height).height = 0;
    let mut width_overflow = scope("scope.png", ImageFormat::Png, VideoScope::Waveform);
    scope_settings(&mut width_overflow).width = i32::MAX as u32 + 1;
    let mut height_overflow = scope("scope.png", ImageFormat::Png, VideoScope::Waveform);
    scope_settings(&mut height_overflow).height = i32::MAX as u32 + 1;
    let mut budget = scope("scope.png", ImageFormat::Png, VideoScope::Waveform);
    scope_settings(&mut budget).width = MAX_DIMENSION + 1;
    let mut frame_budget = scope("scope.png", ImageFormat::Png, VideoScope::Waveform);
    scope_settings(&mut frame_budget).width = 7_680;
    scope_settings(&mut frame_budget).height = 4_320;
    let extension = scope("scope.jpg", ImageFormat::Png, VideoScope::Waveform);
    let jpeg_extension = scope("scope.png", ImageFormat::Jpeg, VideoScope::Waveform);
    let mut end = scope("scope.png", ImageFormat::Png, VideoScope::Waveform);
    scope_settings(&mut end).at = RationalTime::new(600, 600).unwrap();

    for value in [
        width,
        height,
        width_overflow,
        height_overflow,
        budget,
        frame_budget,
        extension,
        jpeg_extension,
        end,
    ] {
        assert_invalid(value, "OUTPUT_SCOPE");
    }

    let value = scope("scope.png", ImageFormat::Png, VideoScope::Waveform);
    let mut empty = project_with(value);
    for track in &mut empty.project.sequences[0].tracks {
        track.clips.clear();
    }
    assert_code(&validation_codes(&empty), "OUTPUT_SCOPE");
}

#[test]
fn scope_time_must_be_nonnegative_safe_and_use_the_project_timebase() {
    for at in [
        RationalTime {
            value: -1,
            timescale: 600,
        },
        RationalTime {
            value: 0,
            timescale: 1,
        },
        RationalTime {
            value: 0,
            timescale: 0,
        },
        RationalTime {
            value: i64::MAX,
            timescale: 600,
        },
    ] {
        let mut value = scope("scope.png", ImageFormat::Png, VideoScope::Waveform);
        scope_settings(&mut value).at = at;
        assert_invalid(value, "OUTPUT_SCOPE_TIME");
    }
}

fn scope_settings(value: &mut Deliverable) -> &mut ScopeOutput {
    let DeliverableKind::Scope(settings) = &mut value.kind else {
        panic!()
    };
    settings
}
