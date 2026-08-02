use super::super::{format_document, lower_document, parse, ArtifactRecipe, AudioMixSourceDecl};
use super::project;

fn document(outputs: &str) -> String {
    project(&format!(
        r#"
  sequence main {{
    layer caption captions {{}}
    layer audio dialogue {{}}
  }}
{outputs}
"#
    ))
}

pub(super) const OUTPUTS: &str = r#"
  delivery preview {
    sequence main;
    raster { canvas 1920px by 1080px; frame-rate 30fps; captions burn-in; }
    artifact video video-out {
      target file "preview.mp4";
      mux mp4 {
        layout fast-start;
        video h264 {
          pixel-format yuv420p;
          alpha opaque;
          rate-control capped {
            target 8mbps;
            max 10mbps;
            buffer 16mbit;
          }
          profile h264-high;
          level "4.1";
        }
        audio aac { sample-rate 48khz; channel-layout stereo; }
        passes two-pass;
        accelerator videotoolbox;
      }
    }
    artifact image-sequence frames {
      target pattern "frame-%d.exr"; numbering from 1001; encode exr;
    }
    artifact caption-sidecar sidecar {
      target file "captions.vtt";
      source caption-tracks { track captions; }
      encode web-vtt;
    }
    artifact audio-stem dialogue-stem {
      target file "dialogue.wav";
      source track dialogue;
      encode wav {
        sample-format pcm-s24le;
        sample-rate 48khz;
        channel-layout stereo;
      }
    }
    artifact scope waveform {
      target file "waveform.png";
      analyze waveform; frame containing 1s; canvas 1280px by 720px; encode png;
    }
  }
"#;

#[test]
fn parses_all_typed_output_variants() {
    let parsed = parse(&document(OUTPUTS)).unwrap();
    assert_eq!(parsed.project.deliveries.len(), 1);
    let artifacts = &parsed.project.deliveries[0].artifacts;
    assert_eq!(artifacts.len(), 5);
    let ArtifactRecipe::Video(video) = &artifacts[0].recipe else {
        panic!("video expected")
    };
    assert!(matches!(
        video.video.rate_control,
        crate::authoring::VideoRateControl::Bitrate {
            target_bps: 8_000_000,
            ..
        }
    ));
    let ArtifactRecipe::AudioStem(stem) = &artifacts[3].recipe else {
        panic!("stem expected")
    };
    assert!(matches!(stem.source, AudioMixSourceDecl::Track(_)));
    let diagnostics = lower_document(&parsed).unwrap_err();
    assert!(!diagnostics.as_slice().is_empty());
}

#[test]
fn delivery_format_is_idempotent() {
    let once = format_document(&parse(&document(OUTPUTS)).unwrap());
    let twice = format_document(&parse(&once).unwrap());
    assert_eq!(once, twice);
    assert!(once.contains("rate-control capped"));
    assert!(once.contains("source track dialogue;"));
    assert!(!once.contains("mode test"));
}

#[test]
fn delivery_schema_rejects_unknown_and_untyped_values() {
    let cases = [
        (
            "delivery bad { sequence main; raster { canvas 1px by 1px; frame-rate 30fps; captions discard; } artifact video bad { target file \"../bad.mp4\"; mux mp4 { video h264 {} audio none; } } }",
            "AUTHORING_ARTIFACT_TARGET_FILE",
        ),
        (
            "delivery bad { sequence main; raster { canvas 2px by 2px; frame-rate 30fps; captions discard; } artifact video bad { target file \"bad.mp4\"; mux mp4 { video h264 {} audio none; } mystery true; } }",
            "AUTHORING_UNKNOWN_FIELD",
        ),
        (
            "delivery bad { sequence main; raster { canvas 2px by 2px; frame-rate 30fps; captions discard; } artifact video bad { target file \"bad.mp4\"; mux mp4 { video h266 {} audio none; } } }",
            "AUTHORING_RECIPE_VARIANT",
        ),
        (
            "delivery bad { sequence main; artifact audio-stem bad { target file \"bad.wav\"; source item wrong; encode wav { sample-format pcm-s24le; sample-rate 48khz; channel-layout stereo; } } }",
            "AUTHORING_OUTPUT_SOURCE",
        ),
        (
            "delivery bad { sequence main; artifact audio-stem bad { target file \"bad.wav\"; encoding {} } }",
            "AUTHORING_LEGACY_ARTIFACT_ENCODING",
        ),
    ];
    for (output, code) in cases {
        let errors = parse(&document(output)).unwrap_err();
        assert!(
            errors.as_slice().iter().any(|value| value.code == code),
            "expected {code}, got {errors:?}"
        );
    }
}
