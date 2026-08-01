use crate::authoring::{format_document, lower_document, parse};

use super::project;

const DELIVERY: &str = r#"
delivery formats {
  sequence main;
  raster { canvas 1280px by 720px; frame-rate 24fps; captions discard; }
  artifact audio-file podcast {
    target file "podcast.mp3";
    source track dialogue;
    encode mp3 { bitrate 192kbps; sample-rate 48khz; channel-layout stereo; }
  }
  artifact animated-image loop {
    target file "loop.gif";
    encode gif { playback 3times; dither floyd-steinberg; }
  }
  artifact still-image cover {
    target file "cover.png";
    frame containing 1s;
    encode png;
  }
  artifact adaptive-package stream {
    target package "stream";
    package hls {
      segment-duration 1s;
      audio {
        source master;
        encode aac { bitrate 192kbps; sample-rate 48khz; channel-layout stereo; }
      }
      rendition hd {
        canvas 1280px by 720px;
        encode h264 {
          rate-control capped { target 3mbps; max 3210kbps; buffer 6mbit; }
        }
      }
    }
  }
}
"#;

#[test]
fn extended_deliveries_round_trip_and_lower_to_distinct_primitives() {
    let parsed = parse(&document(DELIVERY)).unwrap();
    let formatted = format_document(&parsed);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    for evidence in [
        "artifact audio-file podcast",
        "artifact animated-image loop",
        "artifact still-image cover",
        "artifact adaptive-package stream",
        "target package \"stream\";",
        "source track dialogue;",
    ] {
        assert!(formatted.contains(evidence), "missing {evidence}");
    }

    let envelope = lower_document(&parse(&formatted).unwrap()).unwrap();
    let values = &envelope.project.render_configs[0].deliverables;
    assert_eq!(values.len(), 4);
    assert!(values
        .iter()
        .any(|value| matches!(value.kind, veac_ir::DeliverableKind::AudioFile(_))));
    assert!(values
        .iter()
        .any(|value| matches!(value.kind, veac_ir::DeliverableKind::AnimatedImage(_))));
    assert!(values
        .iter()
        .any(|value| matches!(value.kind, veac_ir::DeliverableKind::StillImage(_))));
    assert!(values
        .iter()
        .any(|value| matches!(value.kind, veac_ir::DeliverableKind::AdaptivePackage(_))));
    assert!(values.iter().any(|value| matches!(value.target, veac_ir::DeliverableTarget::Package { ref name } if name == "stream")));
}

#[test]
fn extended_delivery_targets_and_audio_references_are_closed() {
    for (delivery, code) in [
        (
            DELIVERY.replace("target file \"podcast.mp3\"", "target package \"podcast\""),
            "AUTHORING_ARTIFACT_TARGET_KIND",
        ),
        (
            DELIVERY.replace("target package \"stream\"", "target file \"stream.m3u8\""),
            "AUTHORING_ARTIFACT_TARGET_KIND",
        ),
        (
            DELIVERY.replace("source track dialogue", "source track missing"),
            "AUTHORING_REFERENCE_NOT_FOUND",
        ),
        (
            DELIVERY.replace("source master;", "source track missing;"),
            "AUTHORING_REFERENCE_NOT_FOUND",
        ),
    ] {
        let diagnostics = parse(&document(&delivery)).unwrap_err();
        assert!(diagnostics
            .as_slice()
            .iter()
            .any(|value| value.code == code));
    }
}

#[test]
fn hls_rendition_ids_are_unique_at_authoring_time() {
    let delivery = r#"
delivery formats {
  sequence main;
  raster { canvas 1280px by 720px; frame-rate 24fps; captions discard; }
  artifact adaptive-package stream {
    target package "stream";
    package hls {
      segment-duration 1s;
      audio none;
      rendition same { canvas 1280px by 720px; encode h264 { rate-control capped { target 3mbps; max 3210kbps; buffer 6mbit; } } }
      rendition same { canvas 640px by 360px; encode h264 { rate-control capped { target 1mbps; max 1100kbps; buffer 2mbit; } } }
    }
  }
}
"#;
    let diagnostics = parse(&document(delivery)).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_HLS_RENDITION_DUPLICATE"));
}

#[test]
fn audio_file_may_select_audio_carried_by_a_visual_layer() {
    let delivery = DELIVERY.replace("source track dialogue", "source track picture");
    let parsed = parse(&document(&delivery)).unwrap();
    lower_document(&parsed).unwrap();
}

fn document(delivery: &str) -> String {
    project(&format!(
        r#"sequence main {{
  layer video picture {{
    item background {{ source generated solid {{ color #204060ff; }} record {{ at 0s; duration 2s; }} }}
  }}
  layer audio dialogue {{
    item tone {{ source generated silence; record {{ at 0s; duration 2s; }} }}
  }}
}}
{delivery}"#
    ))
}
