use veac_plan::canonical::{Color, TextAnimation};

use super::super::error::TextError;
use super::{all_constant, evaluate, time, Sample, SamplingSpec, UnitSample};

pub(in crate::emitter) fn samples(
    spec: SamplingSpec<'_>,
    animation: Option<&TextAnimation>,
    units: usize,
    frames: usize,
) -> Result<Vec<Sample>, TextError> {
    let seconds = time(spec.duration);
    let Some(animation) = animation else {
        return Ok(vec![constant(&spec, seconds, units, None)?]);
    };
    if all_constant(animation) {
        return Ok(vec![constant(&spec, seconds, units, Some(animation))?]);
    }
    let fps = spec.frame_rate.numerator as f64 / f64::from(spec.frame_rate.denominator);
    let stagger = time(animation.stagger);
    let evaluation = UnitContext {
        spec: &spec,
        animation,
        units,
    };
    let samples = (0..frames)
        .map(|frame| {
            let start = frame as f64 / fps;
            let end = ((frame + 1) as f64 / fps).min(seconds);
            let visible = visible_units(
                evaluate::bound_number(spec.plan, spec.clip, &animation.reveal, start)?,
                units,
            );
            let values = (0..units)
                .map(|unit| {
                    unit_sample(
                        &evaluation,
                        start - unit as f64 * stagger,
                        unit < visible,
                        unit,
                    )
                })
                .collect::<Result<_, _>>()?;
            Ok(Sample {
                start,
                end,
                units: values,
            })
        })
        .collect::<Result<_, TextError>>()?;
    Ok(coalesce(samples))
}

fn constant(
    spec: &SamplingSpec<'_>,
    duration: f64,
    units: usize,
    animation: Option<&TextAnimation>,
) -> Result<Sample, TextError> {
    let visible = match animation {
        Some(value) => visible_units(
            evaluate::bound_number(spec.plan, spec.clip, &value.reveal, 0.0)?,
            units,
        ),
        None => units,
    };
    let evaluation = animation.map(|animation| UnitContext {
        spec,
        animation,
        units,
    });
    let values = (0..units)
        .map(|unit| match &evaluation {
            Some(value) => unit_sample(value, 0.0, unit < visible, unit),
            None => Ok(identity()),
        })
        .collect::<Result<_, _>>()?;
    Ok(Sample {
        start: 0.0,
        end: duration,
        units: values,
    })
}

struct UnitContext<'a> {
    spec: &'a SamplingSpec<'a>,
    animation: &'a TextAnimation,
    units: usize,
}

fn unit_sample(
    context: &UnitContext<'_>,
    seconds: f64,
    visible: bool,
    unit: usize,
) -> Result<UnitSample, TextError> {
    let plan = context.spec.plan;
    let clip = context.spec.clip;
    let animation = context.animation;
    Ok(UnitSample {
        opacity: if visible {
            evaluate::bound_number(plan, clip, &animation.opacity, seconds)?
        } else {
            0.0
        },
        fill_override: highlight_fill(context, seconds, unit)?,
        offset: evaluate::bound_point(
            plan,
            clip,
            &animation.transform.position_offset,
            seconds,
            context.spec.surface,
        )?,
        scale: evaluate::bound_vec2(plan, clip, &animation.transform.scale, seconds)?,
        rotation_degrees: evaluate::bound_number(
            plan,
            clip,
            &animation.transform.rotation_degrees,
            seconds,
        )?,
    })
}

fn highlight_fill(
    context: &UnitContext<'_>,
    seconds: f64,
    unit: usize,
) -> Result<Option<Color>, TextError> {
    let plan = context.spec.plan;
    let clip = context.spec.clip;
    let animation = context.animation;
    let Some(highlight) = animation.highlight.as_ref() else {
        return Ok(None);
    };
    let progress =
        evaluate::bound_number(plan, clip, &highlight.progress, seconds)?.clamp(0.0, 1.0);
    let unit_end = (unit + 1) as f64 / context.units.max(1) as f64;
    Ok((unit_end <= progress).then_some(highlight.fill))
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
