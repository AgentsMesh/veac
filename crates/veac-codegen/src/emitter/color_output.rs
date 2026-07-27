use veac_plan::canonical::ColorSpace;

use super::color_space::{matrix, primaries, range, transfer};

pub(super) fn arguments(value: Option<ColorSpace>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    vec![
        "-color_primaries".to_owned(),
        primaries(value.primaries).to_owned(),
        "-color_trc".to_owned(),
        transfer(value.transfer).to_owned(),
        "-colorspace".to_owned(),
        matrix(value.matrix).to_owned(),
        "-color_range".to_owned(),
        range(value.range).to_owned(),
    ]
}
