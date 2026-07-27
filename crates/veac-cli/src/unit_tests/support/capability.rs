use std::collections::BTreeSet;

pub(super) fn successful_filters() -> BTreeSet<String> {
    [
        "adelay",
        "afade",
        "alphaextract",
        "alphamerge",
        "amix",
        "apad",
        "aresample",
        "atrim",
        "blend",
        "color",
        "concat",
        "crop",
        "fade",
        "format",
        "fps",
        "geq",
        "mergeplanes",
        "negate",
        "overlay",
        "pad",
        "premultiply",
        "scale",
        "setpts",
        "setsar",
        "split",
        "tpad",
        "trim",
        "unpremultiply",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}
