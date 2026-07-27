use std::error::Error;

use crate::{import::native, test_support::*, time, *};

#[test]
fn exact_time_conversion_covers_success_and_failures() {
    assert_eq!(time::from_millis(1500, 1000).unwrap(), time(1500));
    assert_eq!(
        time::range_from_millis(100, 200, 1000).unwrap(),
        range(100, 100)
    );
    assert_eq!(time::to_millis(time(42)).unwrap(), 42);
    assert_eq!(time::range_to_millis(range(5, 10)).unwrap(), (5, 15));
    assert!(time::range_from_millis(2, 1, 1000).is_err());
    assert!(time::from_millis(1, 0).is_err());
    assert!(time::from_millis(1, 24).is_err());
    assert!(time::from_millis(u64::MAX, u32::MAX).is_err());
    assert!(time::from_millis(u64::MAX, 1000).is_err());
    assert!(time::from_millis(veac_ir::MAX_SAFE_INTEGER + 1, 1000).is_err());
    assert!(time::to_millis(veac_ir::RationalTime::new(-1, 1000).unwrap()).is_err());
    assert!(time::to_millis(veac_ir::RationalTime::new(1, 3).unwrap()).is_err());
    let unsafe_end = veac_ir::TimeRange::new(
        veac_ir::RationalTime::new(veac_ir::MAX_SAFE_INTEGER as i64, 1000).unwrap(),
        veac_ir::RationalTime::new(1, 1000).unwrap(),
    )
    .unwrap();
    assert!(time::range_to_millis(unsafe_end).is_err());
}

#[test]
fn errors_are_typed_and_have_sources() {
    let parse = CaptionError::parse(CaptionFormat::Srt, "bad cue");
    assert!(parse.to_string().contains("SRT parse failed"));
    assert!(parse.source().is_none());
    let time = CaptionError::time("inexact");
    assert!(time.to_string().contains("time conversion"));
    let ir = CaptionError::ir("wrong track");
    assert!(ir.to_string().contains("IR conversion"));
    let json: CaptionError = serde_json::from_str::<serde_json::Value>("{")
        .unwrap_err()
        .into();
    assert!(json.source().is_some());
    let validation: CaptionError = validate(&CaptionEnvelope::new(CaptionDocument::new(
        0,
        OverlapPolicy::Reject,
    )))
    .unwrap_err()
    .into();
    assert!(validation.source().is_some());
    assert!(validation.to_string().contains("$.document.timescale"));
}

#[test]
fn identifiers_and_loss_report_are_stable_types() {
    let id = CaptionCueId::new("cap_valid-1").unwrap();
    assert_eq!(id.as_str(), "cap_valid-1");
    assert_eq!(id.to_string(), "cap_valid-1");
    for invalid in ["bad", "cap_", "cap_!", &format!("cap_{}", "a".repeat(125))] {
        assert!(CaptionCueId::new(invalid).is_err());
    }
    assert!(LossReport::default().is_empty());
}

#[test]
fn native_extension_scanners_cover_ignored_blocks_and_defaults() {
    let input = "WEBVTT\n\nNOTE skip\nbody\n\nSTYLE\n::cue{}\n\nREGION\nid:x\n\n00:00:00.000 --> 00:00:01.000\nx\n";
    assert_eq!(native::vtt_ids(input), vec![None]);
    assert!(!native::has_unsupported_vtt_markup(input));
    assert!(!native::has_unsupported_ass_override("{\\b1}x{\\r}"));
    let extras = native::ass_extras(
        "Dialogue: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,x\nDialogue: malformed",
    );
    assert_eq!(extras.len(), 2);
    assert!(extras.iter().all(std::collections::BTreeMap::is_empty));
}
