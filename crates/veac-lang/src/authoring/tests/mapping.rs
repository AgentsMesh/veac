use super::project;
use crate::authoring::{lower_document, parse};
use veac_ir::{PlaybackDirection, SourceOutOfRangePolicy, SourceTimeMap};

fn reverse_source() -> String {
    project(
        r#"
  resource video media {
    locator local { path "media.mov"; }
    streams { video auto; audio disabled; }
  }
  sequence main {
    layer video content {
      item reverse {
        source media resource media;
        record { at 0s; duration 2s; }
        mapping linear { from 4s; to 2s; }
      }
    }
  }
"#,
    )
}

#[test]
fn reverse_linear_lowers_to_the_canonical_source_interval() {
    let envelope = lower_document(&parse(&reverse_source()).unwrap()).unwrap();
    let mapping = envelope.project.sequences[0].tracks[0].clips[0]
        .source_mapping
        .as_ref()
        .unwrap();
    let SourceTimeMap::Linear {
        source_start,
        rate,
        direction,
        repeat,
    } = mapping.time_map
    else {
        panic!("linear mapping expected")
    };
    assert_eq!(source_start.value, 2_000_000);
    assert_eq!(rate.numerator, 1);
    assert_eq!(rate.denominator, 1);
    assert_eq!(direction, PlaybackDirection::Reverse);
    assert_eq!(repeat, 1);
    assert_eq!(mapping.out_of_range, SourceOutOfRangePolicy::Strict);
}

#[test]
fn freeze_rejects_an_outside_policy_it_cannot_preserve() {
    let source = project(
        r#"
  sequence main {
    layer visual content {
      item still {
        source generated solid { color #202020ff; }
        record { at 0s; duration 1s; }
        mapping freeze { source 0s; outside hold-first; }
      }
    }
  }
"#,
    );
    let errors = parse(&source).unwrap_err();
    assert!(errors
        .as_slice()
        .iter()
        .any(|error| error.code == "AUTHORING_MAPPING_FIELD"));
}
