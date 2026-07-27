use crate::authoring::{format_document, lower_document, parse};

pub(super) fn video(video_fields: &str, encoding_fields: &str) -> veac_ir::VideoDeliverable {
    video_result(video_fields, encoding_fields).unwrap_or_else(|error| panic!("{error}"))
}

pub(super) fn video_result(
    video_fields: &str,
    encoding_fields: &str,
) -> Result<veac_ir::VideoDeliverable, crate::authoring::Diagnostics> {
    let extension = ["mov", "mkv", "webm", "mxf"]
        .into_iter()
        .find(|value| encoding_fields.contains(&format!("container {value};")))
        .unwrap_or("mp4");
    let source = format!(
        r#"project output-coverage {{
  settings {{
    timebase 1/1000; canvas 1920px by 1080px;
    frame-rate 30fps; sample-rate 48000hz;
  }}
  entry sequence main;
  sequence main {{
    layer visual picture {{
      item blank {{
        source generated transparent;
        record {{ at 0s; duration 2s; }}
      }}
    }}
  }}
  output video preview {{
    sequence main; file-name "preview.{extension}";
    encoding {{
      video {{ {video_fields} }}
      {encoding_fields}
    }}
  }}
}}"#
    );
    let document = parse(&source).unwrap_or_else(|error| panic!("{error}"));
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    let envelope = lower_document(&document)?;
    let veac_ir::DeliverableKind::Video(video) =
        &envelope.project.render_configs[0].deliverables[0].kind
    else {
        panic!("video deliverable expected")
    };
    Ok(video.clone())
}

pub(super) fn output(kind: &str, body: &str) -> veac_ir::DeliverableKind {
    let file_name = match kind {
        "image-sequence" => {
            let extension = if body.contains("format jpeg;") {
                "jpg"
            } else if body.contains("format tiff;") {
                "tiff"
            } else if body.contains("format exr;") {
                "exr"
            } else {
                "png"
            };
            format!("frame-%06d.{extension}")
        }
        "caption-sidecar" => {
            let extension = if body.contains("format web-vtt;") {
                "vtt"
            } else if body.contains("format ass;") {
                "ass"
            } else {
                "srt"
            };
            format!("captions.{extension}")
        }
        "audio-stem" => {
            let extension = if body.contains("format flac;") {
                "flac"
            } else {
                "wav"
            };
            format!("stem.{extension}")
        }
        "scope" => {
            let extension = if body.contains("format jpeg;") {
                "jpg"
            } else if body.contains("format tiff;") {
                "tiff"
            } else if body.contains("format exr;") {
                "exr"
            } else {
                "png"
            };
            format!("scope.{extension}")
        }
        _ => panic!("unexpected output kind"),
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
  output {kind} result {{ sequence main; file-name "{file_name}"; encoding {{ {body} }} }}
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
