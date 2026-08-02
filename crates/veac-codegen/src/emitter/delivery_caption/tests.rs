use super::*;
use crate::unit_tests::emitter_tests::support::{resolved, text_fixture};
use veac_plan::canonical::{CaptionSidecarFormat, RationalTime, TrackId};

#[test]
fn caption_render_reports_exact_range_overflow_as_an_invalid_time() {
    let mut plan = resolved(&text_fixture(true));
    let track = &mut plan.sequences[0].tracks[1];
    track.clips[0].record_range.start = RationalTime {
        value: veac_plan::canonical::MAX_SAFE_INTEGER as i64,
        timescale: 1,
    };
    track.clips[0].record_range.duration = RationalTime::new(1, 1).unwrap();
    let settings = CaptionSidecarOutput {
        format: CaptionSidecarFormat::Srt,
        track_ids: vec![track.id.clone()],
    };
    let error = render(&plan, &ExecutionBindings::default(), &settings).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "CAPTION_TIME_INVALID");
}

#[test]
fn caption_render_defends_missing_tracks_and_inexact_ass_time() {
    let mut plan = resolved(&text_fixture(true));
    let track_id = plan.sequences[0].tracks[1].id.clone();
    let settings = CaptionSidecarOutput {
        format: CaptionSidecarFormat::Ass,
        track_ids: vec![TrackId::new("trk_absent").unwrap()],
    };
    let error = render(&plan, &ExecutionBindings::default(), &settings).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "CAPTION_TRACK_MISSING");

    plan.sequences[0].tracks[1].clips[0].record_range.start = RationalTime::new(601, 600).unwrap();
    let settings = CaptionSidecarOutput {
        format: CaptionSidecarFormat::Ass,
        track_ids: vec![track_id],
    };
    let error = render(&plan, &ExecutionBindings::default(), &settings).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "CAPTION_TIME_INEXACT");

    let settings = CaptionSidecarOutput {
        format: CaptionSidecarFormat::WebVtt,
        track_ids: settings.track_ids,
    };
    let error = render(&plan, &ExecutionBindings::default(), &settings).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "CAPTION_TIME_INEXACT");
}

#[test]
fn owned_caption_failure_messages_keep_their_typed_contract() {
    let failure = super::failure::Failure::invalid(
        "CAPTION_OWNED_FAILURE",
        String::from("owned caption failure"),
    );
    assert_eq!(failure.kind, CodegenErrorKind::InvalidPlan);
    assert_eq!(failure.message, "owned caption failure");
}
