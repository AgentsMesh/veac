use crate::authoring::{format_document, lower_document, parse};

pub(in crate::authoring::tests) fn video(
    video_fields: &str,
    mux_fields: &str,
) -> veac_ir::VideoDeliverable {
    video_result(video_fields, mux_fields).unwrap_or_else(|error| panic!("{error}"))
}

pub(in crate::authoring::tests) fn video_result(
    video_fields: &str,
    mux_fields: &str,
) -> Result<veac_ir::VideoDeliverable, crate::authoring::Diagnostics> {
    let container = field(mux_fields, "container").unwrap_or("mp4");
    let codec = field(video_fields, "codec").unwrap_or("h264");
    let extension = container;
    let video_fields = video_recipe(video_fields, codec);
    let audio = audio_recipe(mux_fields);
    let layout = if mux_fields.contains("optimize-for-streaming true;") {
        "fast-start"
    } else {
        "standard"
    };
    let passes = field(mux_fields, "pass-mode").unwrap_or("single");
    let accelerator = field(mux_fields, "hardware").unwrap_or("auto");
    let source = format!(
        r#"project output-coverage {{
  settings {{ timebase 1/1000; canvas 1920px by 1080px; frame-rate 30fps; sample-rate 48000hz; }}
  entry sequence main;
  sequence main {{
    layer visual picture {{
      item blank {{ source generated transparent; record {{ at 0s; duration 2s; }} }}
    }}
  }}
  delivery preview {{
    sequence main;
    raster {{ canvas 1920px by 1080px; frame-rate 30fps; captions burn-in; }}
    artifact video preview {{
      target file "preview.{extension}";
      mux {container} {{
        layout {layout};
        video {codec} {{ {video_fields} }}
        {audio}
        passes {passes};
        accelerator {accelerator};
      }}
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

fn video_recipe(source: &str, codec: &str) -> String {
    let mut value = remove_field(source, "codec");
    value = value.replace("gop-size", "gop");
    value = value.replace("rate-control bitrate", "rate-control capped");
    value = unit_field(value, "target-bps", "target", "bps");
    value = unit_field(value, "max-bps", "max", "bps");
    value = unit_field(value, "buffer-bps", "buffer", "bit");
    if codec == "prores" && !value.contains("alpha ") {
        value.push_str(" alpha straight;");
    }
    value
}

fn audio_recipe(source: &str) -> String {
    if source.contains("audio none;") {
        return "audio none;".into();
    }
    let Some(start) = source.find("audio {") else {
        return "audio aac { sample-rate 48khz; channel-layout stereo; }".into();
    };
    let body = &source[start + "audio {".len()..];
    let body = body.split('}').next().unwrap_or("");
    let codec = field(body, "codec").unwrap_or("aac");
    let rate = field(body, "sample-rate").unwrap_or("48000");
    let channels = field(body, "channels").unwrap_or("2");
    format!(
        "audio {codec} {{ sample-rate {rate}hz; channel-layout {}; }}",
        layout(channels)
    )
}

fn layout(channels: &str) -> &'static str {
    match channels {
        "1" => "mono",
        "2" => "stereo",
        "3" => "discrete-3",
        "4" => "discrete-4",
        "5" => "discrete-5",
        "6" => "surround-5-1",
        "7" => "discrete-7",
        "8" => "surround-7-1",
        _ => "invalid",
    }
}

fn field<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let start = source.find(&format!("{name} "))? + name.len() + 1;
    Some(source[start..].split(';').next()?.trim())
}

fn remove_field(source: &str, name: &str) -> String {
    let Some(start) = source.find(&format!("{name} ")) else {
        return source.to_owned();
    };
    let Some(end) = source[start..].find(';') else {
        return source.to_owned();
    };
    format!("{}{}", &source[..start], &source[start + end + 1..])
}

fn unit_field(mut source: String, old: &str, new: &str, unit: &str) -> String {
    let Some(start) = source.find(&format!("{old} ")) else {
        return source;
    };
    let value_start = start + old.len() + 1;
    let Some(end) = source[value_start..].find(';') else {
        return source;
    };
    let value = source[value_start..value_start + end].trim();
    source.replace_range(start..value_start + end, &format!("{new} {value}{unit}"));
    source
}
