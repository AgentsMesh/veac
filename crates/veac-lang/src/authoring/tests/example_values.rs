const GROUPS: &[&[&str]] = &[
    &["true", "false"],
    &[
        "hold",
        "linear",
        "bezier",
        "ease-in",
        "ease-out",
        "ease-in-out",
    ],
    &["clamp", "freeze", "loop", "ping-pong", "transparent"],
    &["left", "center", "right", "top", "middle", "bottom"],
    &[
        "normal", "add", "multiply", "screen", "overlay", "darken", "lighten",
    ],
    &["union", "intersect", "subtract", "xor"],
    &["rectangle", "ellipse", "polygon", "star"],
    &["solid", "linear-gradient", "radial-gradient", "shape"],
    &[
        "marker",
        "chapter",
        "review-note",
        "beat-grid",
        "analysis",
        "custom",
    ],
    &["blur", "mosaic", "solid-fill", "clone", "inpaint"],
    &["replace", "before", "after", "inside-start", "inside-end"],
    &[
        "video",
        "audio",
        "image",
        "generator",
        "compound",
        "caption",
    ],
    &["layer", "composite-band", "item-set"],
    &["premultiplied", "straight", "opaque"],
    &["alpha", "luma"],
    &["source", "timeline", "wall-clock"],
    &["absolute", "relative"],
    &["frame", "field"],
    &["progressive", "interlaced"],
    &["bt601", "bt709", "bt2020"],
    &["srgb", "bt709", "pq", "hlg"],
    &["rgb", "bt601", "bt709", "bt2020-ncl"],
    &["full", "limited"],
    &["center", "left", "top-left"],
    &["h264", "h265", "prores", "vp9", "av1", "dnxhr", "copy"],
    &["aac", "opus", "pcm", "flac", "copy"],
    &[
        "yuv420p",
        "yuv420p10le",
        "yuv422p10le",
        "yuv444p10le",
        "rgb24",
        "rgba",
    ],
    &["lossless", "crf", "bitrate"],
    &["burn-in", "sidecar"],
    &["srt", "webvtt", "ass", "ttml"],
    &["mp4", "mov", "mkv", "webm", "wav", "mp3", "m4a"],
    &[
        "h264-baseline",
        "h264-main",
        "h264-high",
        "h265-main",
        "h265-main10",
        "prores-proxy",
        "prores-lt",
        "prores-422",
        "prores-hq",
        "prores-4444",
    ],
    &["videotoolbox", "cuda", "qsv", "vaapi"],
    &["png", "jpeg", "webp", "tiff", "exr"],
    &["opaque", "straight", "premultiplied"],
    &["smpte170m", "bt709", "bt2020"],
    &["bt709", "srgb", "smpte2084", "arib-std-b67"],
    &["smpte170m", "bt709", "bt2020-ncl"],
    &["source", "sequence", "custom"],
    &["full-mix", "dialogue", "music", "effects", "track", "bus"],
    &["single", "segment"],
    &["track", "bus", "mix"],
    &["crop", "letterbox", "stretch"],
    &["nearest", "bilinear", "bicubic", "lanczos"],
    &["mono", "stereo", "surround-5.1", "surround-7.1"],
    &["source", "sequence", "project"],
    &["whole", "line", "word", "grapheme"],
    &["opacity", "effect", "color"],
];

pub(super) fn alternatives(token: &str) -> Vec<String> {
    if let Some(values) = GROUPS.iter().find(|values| values.contains(&token)) {
        return values
            .iter()
            .filter(|value| **value != token)
            .map(|value| (*value).to_owned())
            .collect();
    }
    if token.starts_with('"') {
        return vec!["\"alternate\"".into(), "\"\"".into()];
    }
    if token.starts_with('#') {
        return vec!["#000000".into(), "#FFFFFFFF".into(), "#12".into()];
    }
    numeric_alternatives(token)
}

fn numeric_alternatives(token: &str) -> Vec<String> {
    let Some(split) = token
        .char_indices()
        .skip_while(|(_, value)| value.is_ascii_digit() || ".+-".contains(*value))
        .map(|(index, _)| index)
        .next()
        .or_else(|| token.parse::<f64>().ok().map(|_| token.len()))
    else {
        return Vec::new();
    };
    if token[..split].parse::<f64>().is_err() {
        return Vec::new();
    }
    let suffix = &token[split..];
    ["-1", "0", "1", "1.5", "999999999999999999999"]
        .into_iter()
        .filter(|value| *value != &token[..split])
        .map(|value| format!("{value}{suffix}"))
        .collect()
}
