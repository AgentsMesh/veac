use crate::authoring::{format_document, lower_document, parse};

use super::project;

const DELIVERY: &str = r#"
  delivery preview {
    sequence main;
    raster { canvas 1280px by 720px; frame-rate 24fps; captions burn-in; }
    artifact video master {
      target file "master.mp4";
      mux mp4 { video h264 {} audio none; }
    }
    artifact image-sequence frames {
      target pattern "frame-%04d.png";
      numbering from 1; encode png;
    }
    artifact audio-stem mix {
      target file "mix.wav";
      source master;
      encode wav { sample-format pcm-s24le; sample-rate 48khz; channel-layout stereo; }
    }
  }
"#;

#[test]
fn delivery_round_trips_and_lowers_shared_raster_once() {
    let source = document(DELIVERY);
    let parsed = parse(&source).unwrap();
    let formatted = format_document(&parsed);
    let reparsed = parse(&formatted).unwrap();
    assert_eq!(format_document(&reparsed), formatted);

    let envelope = lower_document(&reparsed).unwrap();
    let config = &envelope.project.render_configs[0];
    assert_eq!(config.deliverables.len(), 3);
    let raster = config.raster.as_ref().unwrap();
    assert_eq!((raster.width, raster.height), (1280, 720));
    assert_eq!(raster.frame_rate, veac_ir::Rational::new(24, 1).unwrap());
    assert_eq!(raster.captions, veac_ir::CaptionOutput::BurnIn);
    let target = |id| {
        &config
            .deliverables
            .iter()
            .find(|value| value.id.as_str() == id)
            .unwrap()
            .target
    };
    assert!(matches!(
        target("dlv_master"),
        veac_ir::DeliverableTarget::File { .. }
    ));
    assert!(matches!(
        target("dlv_frames"),
        veac_ir::DeliverableTarget::ImageSequence { .. }
    ));
}

#[test]
fn delivery_raster_presence_matches_artifact_domains() {
    for (delivery, code) in [
        (
            "delivery no-raster { sequence main; artifact video v { target file \"v.mp4\"; mux mp4 { video h264 {} audio none; } } }",
            "AUTHORING_DELIVERY_RASTER_REQUIRED",
        ),
        (
            "delivery unused { sequence main; raster { canvas 1px by 1px; frame-rate 1fps; captions discard; } artifact audio-stem a { target file \"a.wav\"; source master; encode wav { sample-format pcm-s24le; sample-rate 48khz; channel-layout stereo; } } }",
            "AUTHORING_DELIVERY_RASTER_UNUSED",
        ),
    ] {
        assert_parse_code(delivery, code);
    }
}

#[test]
fn artifact_kind_and_target_are_exactly_paired() {
    for (delivery, code) in [
        (
            "delivery bad { sequence main; raster { canvas 1px by 1px; frame-rate 1fps; captions discard; } artifact video v { target pattern \"v-%d.mp4\"; mux mp4 { video h264 {} audio none; } } }",
            "AUTHORING_ARTIFACT_TARGET_KIND",
        ),
        (
            "delivery bad { sequence main; raster { canvas 1px by 1px; frame-rate 1fps; captions discard; } artifact image-sequence frames { target file \"frame.png\"; numbering from 1; encode png; } }",
            "AUTHORING_ARTIFACT_TARGET_KIND",
        ),
        (
            "delivery bad { sequence main; raster { canvas 1px by 1px; frame-rate 1fps; captions discard; } artifact image-sequence frames { target pattern \"frame.png\"; numbering from 1; encode png; } }",
            "AUTHORING_ARTIFACT_TARGET_PATTERN",
        ),
    ] {
        assert_parse_code(delivery, code);
    }
}

fn assert_parse_code(delivery: &str, code: &str) {
    let diagnostics = parse(&document(delivery)).unwrap_err();
    assert!(
        diagnostics
            .as_slice()
            .iter()
            .any(|value| value.code == code),
        "expected {code}, got {diagnostics:?}"
    );
}

fn document(delivery: &str) -> String {
    project(&format!(
        r#"sequence main {{
  layer audio dialogue {{
    item tone {{
      source generated silence;
      record {{ at 0s; duration 1s; }}
    }}
  }}
}}
{delivery}"#
    ))
}
