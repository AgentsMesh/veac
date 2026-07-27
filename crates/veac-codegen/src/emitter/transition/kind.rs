mod expressions;

use veac_plan::canonical::{FadeColor, TransitionKind};

pub(super) fn filter(value: &TransitionKind, duration: &str) -> String {
    let native = match value {
        TransitionKind::Dissolve => Some("fade"),
        TransitionKind::Fade { color } => Some(match color {
            FadeColor::Transparent => "fade",
            FadeColor::Black => "fadeblack",
            FadeColor::White => "fadewhite",
        }),
        _ => None,
    };
    match native {
        Some(name) => format!("xfade=transition={name}:duration={duration}:offset=0"),
        None => format!(
            "xfade=transition=custom:duration={duration}:offset=0:expr='{}'",
            expressions::custom(value)
        ),
    }
}
