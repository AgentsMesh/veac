use veac_plan::canonical::ColorSpace;

use super::color_space::{matrix, primaries, range, transfer};

pub(super) fn arguments(value: Option<ColorSpace>) -> Vec<String> {
    stream_arguments(value, "")
}

pub(super) fn stream_arguments(value: Option<ColorSpace>, suffix: &str) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    vec![
        option("-color_primaries", suffix),
        primaries(value.primaries).to_owned(),
        option("-color_trc", suffix),
        transfer(value.transfer).to_owned(),
        option("-colorspace", suffix),
        matrix(value.matrix).to_owned(),
        option("-color_range", suffix),
        range(value.range).to_owned(),
    ]
}

fn option(name: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        name.to_owned()
    } else {
        format!("{name}:v{suffix}")
    }
}
