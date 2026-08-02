use crate::authoring::{format_document, lower_document, parse};

use super::project;

fn source(mapping: &str) -> String {
    project(&format!(
        r#"resource video media {{
  locator local {{ path "media.mov"; }}
  streams {{ video auto; audio disabled; }}
}}
sequence main {{
  layer visual picture {{
    item blank {{
      source media resource media;
      record {{ at 0s; duration 4s; }}
      {mapping}
    }}
  }}
}}"#
    ))
}

fn clip(mapping: &str) -> veac_ir::Clip {
    let document = parse(&source(mapping)).unwrap_or_else(|error| panic!("{error}"));
    let envelope = lower_document(&document).unwrap_or_else(|error| panic!("{error}"));
    envelope.project.sequences[0].tracks[0].clips[0].clone()
}

fn mapping(authored: &str) -> veac_ir::SourceMapping {
    let clip = clip(authored);
    clip.source_mapping
        .clone()
        .unwrap_or_else(|| panic!("missing mapping on {clip:?}"))
}

fn lower_error(mapping: &str, code: &str, message: &str) {
    let document = parse(&source(mapping)).unwrap_or_else(|error| panic!("{error}"));
    let error = lower_document(&document).unwrap_err();
    assert!(
        error
            .as_slice()
            .iter()
            .any(|value| value.code == code && value.message.contains(message)),
        "unexpected diagnostic: {error}"
    );
}

#[test]
fn forward_and_reverse_linear_mappings_preserve_policy_rate_and_direction() {
    let forward = mapping("mapping linear { from 1s; to 3s; outside hold-first; }");
    assert_eq!(
        forward.out_of_range,
        veac_ir::SourceOutOfRangePolicy::HoldFirst
    );
    let veac_ir::SourceTimeMap::Linear {
        source_start,
        rate,
        direction,
        ..
    } = forward.time_map
    else {
        panic!("linear map expected")
    };
    assert_eq!(
        (source_start.value, rate.numerator, rate.denominator),
        (1_000_000, 1, 2)
    );
    assert_eq!(direction, veac_ir::PlaybackDirection::Forward);

    let reverse = mapping("mapping linear { from 5s; to 1s; outside hold-last; }");
    assert_eq!(
        reverse.out_of_range,
        veac_ir::SourceOutOfRangePolicy::HoldLast
    );
    let veac_ir::SourceTimeMap::Linear {
        source_start,
        rate,
        direction,
        ..
    } = reverse.time_map
    else {
        panic!("linear map expected")
    };
    assert_eq!(
        (source_start.value, rate.numerator, rate.denominator),
        (1_000_000, 1, 1)
    );
    assert_eq!(direction, veac_ir::PlaybackDirection::Reverse);
}

#[test]
fn freeze_and_curve_mappings_lower_to_exact_segments() {
    let frozen = clip("mapping freeze { source 1500ms; }");
    assert!(frozen.source_mapping.is_none());
    let veac_ir::ClipSource::FreezeFrame { source_time, .. } = frozen.source else {
        panic!("freeze-frame source expected")
    };
    assert_eq!(
        (source_time.value, source_time.timescale),
        (1_500_000, 1_000_000)
    );

    let curved = mapping(
        r#"mapping curve {
          outside hold-both;
          key a { at 0s; source 1s; interpolation linear; }
          key b { at 1s; source 2s; interpolation linear; }
          key c { at 4s; source 8s; interpolation linear; }
        }"#,
    );
    assert_eq!(
        curved.out_of_range,
        veac_ir::SourceOutOfRangePolicy::HoldBoth
    );
    let veac_ir::SourceTimeMap::Curve { segments } = curved.time_map else {
        panic!("curve map expected")
    };
    assert_eq!(segments.len(), 2);
    assert_eq!(
        (
            segments[0].record_duration.value,
            segments[1].record_duration.value
        ),
        (1_000_000, 3_000_000)
    );
    assert_eq!(
        segments[0].interpolation,
        veac_ir::SourceTimeInterpolation::Linear
    );
    assert_eq!(
        segments[1].interpolation,
        veac_ir::SourceTimeInterpolation::Linear
    );
}

#[test]
fn invalid_mapping_ranges_and_interpolation_report_lowering_diagnostics() {
    lower_error(
        "mapping curve { key a { at 0s; source 0s; interpolation hold; } key b { at 4s; source 4s; interpolation linear; } }",
        "AUTHORING_LOWER_IR_VALIDATION",
        "SOURCE_TIME_INTERPOLATION",
    );
    lower_error(
        "mapping curve { key only { at 0s; source 0s; interpolation linear; } }",
        "AUTHORING_LOWER_MAPPING",
        "at least two keys",
    );
    lower_error(
        "mapping curve { key a { at 1s; source 1s; interpolation linear; } key b { at 4s; source 4s; interpolation linear; } }",
        "AUTHORING_LOWER_MAPPING_DOMAIN",
        "complete item record duration",
    );
    lower_error(
        "mapping curve { key a { at 0s; source 0s; interpolation ease-in; } key b { at 4s; source 4s; interpolation linear; } }",
        "AUTHORING_LOWER_UNSUPPORTED",
        "eased source-time interpolation",
    );
}

#[test]
fn formatter_emits_every_leaf_mapping_interpolation_keyword() {
    let document = parse(&source(
        r#"mapping curve {
          key a { at 0s; source 0s; interpolation hold; }
          key b { at 1s; source 1s; interpolation linear; }
          key c { at 2s; source 2s; interpolation ease-in; }
          key d { at 3s; source 3s; interpolation ease-out; }
          key e { at 4s; source 4s; interpolation ease-in-out; }
        }"#,
    ))
    .unwrap();
    let formatted = format_document(&document);
    for keyword in ["hold", "linear", "ease-in", "ease-out", "ease-in-out"] {
        assert!(formatted.contains(&format!("interpolation {keyword};")));
    }
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
}
