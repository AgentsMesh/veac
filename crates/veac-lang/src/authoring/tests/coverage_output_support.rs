use crate::authoring::{format_document, lower_document, parse};

#[path = "coverage_output_support/video.rs"]
mod video_helpers;

pub(super) use video_helpers::{video, video_result};

pub(super) fn output(kind: &str, body: &str) -> veac_ir::DeliverableKind {
    let target = match kind {
        "image-sequence" => {
            let extension = if body.contains("encode jpeg;") {
                "jpg"
            } else if body.contains("encode tiff;") {
                "tiff"
            } else if body.contains("encode exr;") {
                "exr"
            } else {
                "png"
            };
            format!("pattern \"frame-%06d.{extension}\"")
        }
        "caption-sidecar" => {
            let extension = if body.contains("encode web-vtt;") {
                "vtt"
            } else if body.contains("encode ass;") {
                "ass"
            } else {
                "srt"
            };
            format!("file \"captions.{extension}\"")
        }
        "audio-stem" => {
            let extension = if body.contains("encode flac") {
                "flac"
            } else {
                "wav"
            };
            format!("file \"stem.{extension}\"")
        }
        "scope" => {
            let extension = if body.contains("encode jpeg;") {
                "jpg"
            } else if body.contains("encode tiff;") {
                "tiff"
            } else if body.contains("encode exr;") {
                "exr"
            } else {
                "png"
            };
            format!("file \"scope.{extension}\"")
        }
        _ => panic!("unexpected output kind"),
    };
    let raster = if matches!(kind, "image-sequence" | "scope") {
        "raster { canvas 1920px by 1080px; frame-rate 30fps; captions burn-in; }"
    } else {
        ""
    };
    let source = format!(
        r#"project output-coverage {{
  settings {{
    timebase 1/1000; canvas 1920px by 1080px;
    frame-rate 30fps; sample-rate 48000hz;
  }}
  entry sequence main;
  sequence main {{
    layer audio dialogue {{ route bus dialogue; }}
    layer caption captions {{
      item line {{ source caption {{ content "Line"; }} record {{ at 0s; duration 1s; }} }}
    }}
  }}
  delivery result {{
    sequence main;
    {raster}
    artifact {kind} result {{ target {target}; {body} }}
  }}
}}"#
    );
    let document = parse(&source).unwrap_or_else(|error| panic!("{error}"));
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope = lower_document(&document).unwrap_or_else(|error| panic!("{error}"));
    envelope.project.render_configs[0].deliverables[0]
        .kind
        .clone()
}
