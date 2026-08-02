mod evaluate;

use veac_plan::canonical::{Animatable, Color, Rational, RationalTime, TextAnimation};

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

pub(super) fn samples(
    animation: Option<&TextAnimation>,
    duration: RationalTime,
    frame_rate: Rational,
    units: usize,
    surface: (u32, u32),
    frames: usize,
) -> Vec<Sample> {
    let seconds = time(duration);
    let Some(animation) = animation else {
        return vec![constant(seconds, units, None, surface)];
    };
    if all_constant(animation) {
        return vec![constant(seconds, units, Some(animation), surface)];
    }
    let fps = frame_rate.numerator as f64 / f64::from(frame_rate.denominator);
    let stagger = time(animation.stagger);
    let samples = (0..frames)
        .map(|frame| {
            let start = frame as f64 / fps;
            let end = ((frame + 1) as f64 / fps).min(seconds);
            let visible = visible_units(evaluate::number(&animation.reveal, start), units);
            let values = (0..units)
                .map(|unit| {
                    unit_sample(
                        animation,
                        start - unit as f64 * stagger,
                        unit < visible,
                        unit,
                        units,
                        surface,
                    )
                })
                .collect();
            Sample {
                start,
                end,
                units: values,
            }
        })
        .collect();
    coalesce(samples)
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

fn constant(
    duration: f64,
    units: usize,
    animation: Option<&TextAnimation>,
    surface: (u32, u32),
) -> Sample {
    let visible = animation.map_or(units, |value| {
        visible_units(evaluate::number(&value.reveal, 0.0), units)
    });
    let values = (0..units)
        .map(|unit| match animation {
            Some(value) => unit_sample(value, 0.0, unit < visible, unit, units, surface),
            None => identity(),
        })
        .collect();
    Sample {
        start: 0.0,
        end: duration,
        units: values,
    }
}

fn unit_sample(
    animation: &TextAnimation,
    seconds: f64,
    visible: bool,
    unit: usize,
    units: usize,
    surface: (u32, u32),
) -> UnitSample {
    UnitSample {
        opacity: if visible {
            evaluate::number(&animation.opacity, seconds)
        } else {
            0.0
        },
        fill_override: highlight_fill(animation, seconds, unit, units),
        offset: evaluate::point(&animation.transform.position_offset, seconds, surface),
        scale: evaluate::vec2(&animation.transform.scale, seconds),
        rotation_degrees: evaluate::number(&animation.transform.rotation_degrees, seconds),
    }
}

fn identity() -> UnitSample {
    UnitSample {
        opacity: 1.0,
        fill_override: None,
        offset: (0.0, 0.0),
        scale: (1.0, 1.0),
        rotation_degrees: 0.0,
    }
}

fn all_constant(value: &TextAnimation) -> bool {
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

fn highlight_fill(
    animation: &TextAnimation,
    seconds: f64,
    unit: usize,
    units: usize,
) -> Option<Color> {
    let highlight = animation.highlight.as_ref()?;
    let progress = evaluate::number(&highlight.progress, seconds).clamp(0.0, 1.0);
    let unit_end = (unit + 1) as f64 / units.max(1) as f64;
    (unit_end <= progress).then_some(highlight.fill)
}

fn coalesce(samples: Vec<Sample>) -> Vec<Sample> {
    let mut merged: Vec<Sample> = Vec::with_capacity(samples.len());
    for sample in samples {
        if let Some(previous) = merged
            .last_mut()
            .filter(|value| value.units == sample.units)
        {
            previous.end = sample.end;
        } else {
            merged.push(sample);
        }
    }
    merged
}

fn visible_units(reveal: f64, units: usize) -> usize {
    ((reveal.clamp(0.0, 1.0) * units as f64) + 1e-9)
        .floor()
        .min(units as f64) as usize
}

fn time(value: RationalTime) -> f64 {
    value.value as f64 / f64::from(value.timescale)
}
