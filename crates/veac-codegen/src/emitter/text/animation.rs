mod evaluate;
mod sampling;

pub(super) use sampling::samples;
use veac_plan::canonical::{Animatable, Color, Rational, RationalTime, TextAnimation};

pub(super) struct SamplingSpec<'a> {
    pub(super) plan: &'a veac_plan::ResolvedRenderPlan,
    pub(super) clip: &'a veac_plan::ResolvedClip,
    pub(super) duration: RationalTime,
    pub(super) frame_rate: Rational,
    pub(super) surface: (u32, u32),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct UnitSample {
    pub opacity: f64,
    pub fill_override: Option<Color>,
    pub offset: (f64, f64),
    pub scale: (f64, f64),
    pub rotation_degrees: f64,
}

#[derive(Debug)]
pub(super) struct Sample {
    pub start: f64,
    pub end: f64,
    pub units: Vec<UnitSample>,
}

pub(super) fn sample_count(
    animation: Option<&TextAnimation>,
    duration: RationalTime,
    frame_rate: Rational,
) -> Option<usize> {
    if animation.is_none_or(all_constant) {
        return Some(1);
    }
    let numerator = i128::from(duration.value).checked_mul(i128::from(frame_rate.numerator))?;
    let denominator =
        i128::from(duration.timescale).checked_mul(i128::from(frame_rate.denominator))?;
    if numerator <= 0 || denominator <= 0 {
        return None;
    }
    let rounded = numerator.checked_add(denominator - 1)? / denominator;
    usize::try_from(rounded.max(1)).ok()
}

pub(super) fn has_transform(animation: Option<&TextAnimation>) -> bool {
    animation.is_some_and(|value| {
        !matches!(&value.transform.position_offset, Animatable::Constant { value } if value.x.value == 0.0 && value.y.value == 0.0)
            || !matches!(&value.transform.scale, Animatable::Constant { value } if value.x == 1.0 && value.y == 1.0)
            || !matches!(&value.transform.rotation_degrees, Animatable::Constant { value } if *value == 0.0)
    })
}

pub(super) fn all_constant(value: &TextAnimation) -> bool {
    evaluate::constant(&value.reveal)
        && value
            .highlight
            .as_ref()
            .is_none_or(|highlight| evaluate::constant(&highlight.progress))
        && evaluate::constant(&value.opacity)
        && evaluate::constant(&value.transform.position_offset)
        && evaluate::constant(&value.transform.scale)
        && evaluate::constant(&value.transform.rotation_degrees)
}

pub(super) fn time(value: RationalTime) -> f64 {
    value.value as f64 / f64::from(value.timescale)
}
