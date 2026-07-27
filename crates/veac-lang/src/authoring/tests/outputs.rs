use super::super::{format_document, lower_document, parse, AudioStemSourceDecl, OutputEncoding};
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
  output video preview {
    sequence main;
    file-name "preview.mp4";
    encoding {
      container mp4;
      video {
        codec h264;
        pixel-format yuv420p;
        alpha opaque;
        rate-control bitrate {
          target-bps 8000000;
          max-bps 10000000;
          buffer-bps 16000000;
        }
        profile h264-high;
        level "4.1";
      }
      audio { codec aac; sample-rate 48000; channels 2; }
      captions burn-in;
      optimize-for-streaming true;
      pass-mode two-pass;
      hardware videotoolbox;
    }
  }
  output image-sequence frames {
    sequence main; file-name "frame-%d.exr";
    encoding { format exr; start-number 1001; }
  }
  output caption-sidecar sidecar {
    sequence main; file-name "captions.vtt";
    encoding { format web-vtt; tracks { track captions; } }
  }
  output audio-stem dialogue-stem {
    sequence main; file-name "dialogue.wav";
    encoding {
      format wav;
      audio { codec pcm-s24le; sample-rate 48000; channels 2; }
      source track dialogue;
    }
  }
  output scope waveform {
    sequence main; file-name "waveform.png";
    encoding { scope waveform; at 1s; width 1280; height 720; format png; }
  }
"#;

#[test]
fn parses_all_typed_output_variants() {
    let parsed = parse(&document(OUTPUTS)).unwrap();
    assert_eq!(parsed.project.outputs.len(), 5);
    let OutputEncoding::Video(video) = &parsed.project.outputs[0].encoding else {
        panic!("video expected")
    };
    assert!(matches!(
        video.video.rate_control,
        crate::authoring::VideoRateControl::Bitrate {
            target_bps: 8_000_000,
            ..
        }
    ));
    let OutputEncoding::AudioStem(stem) = &parsed.project.outputs[3].encoding else {
        panic!("stem expected")
    };
    assert!(matches!(stem.source, AudioStemSourceDecl::Track(_)));
    let diagnostics = lower_document(&parsed).unwrap_err();
    assert!(!diagnostics.as_slice().is_empty());
}

#[test]
fn output_format_is_idempotent() {
    let once = format_document(&parse(&document(OUTPUTS)).unwrap());
    let twice = format_document(&parse(&once).unwrap());
    assert_eq!(once, twice);
    assert!(once.contains("rate-control bitrate"));
    assert!(once.contains("source track dialogue;"));
    assert!(!once.contains("mode test"));
}

#[test]
fn output_schema_rejects_unknown_and_untyped_values() {
    let cases = [
        (
            "output video bad { sequence main; file-name \"../bad.mp4\"; encoding {} }",
            "AUTHORING_OUTPUT_FILE_NAME",
        ),
        (
            "output video bad { sequence main; file-name \"bad.mp4\"; encoding { mystery true; } }",
            "AUTHORING_UNKNOWN_FIELD",
        ),
        (
            "output video bad { sequence main; file-name \"bad.mp4\"; encoding { video { codec h266; } } }",
            "AUTHORING_OUTPUT_ENUM",
        ),
        (
            "output audio-stem bad { sequence main; file-name \"bad.wav\"; encoding { source item wrong; } }",
            "AUTHORING_OUTPUT_SOURCE",
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
