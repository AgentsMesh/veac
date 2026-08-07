use veac_plan::canonical::{PlaybackDirection, SourceTimeInterpolation};
use veac_plan::{ResolvedClip, ResolvedSourceTimeMap};

use super::budget::Budget;
use super::error::TemporalBackendError;
use super::expression;
use super::value::{number, Expression};

pub(super) fn compile(
    clip: &ResolvedClip,
    local: &Expression,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let mapping = clip.source_mapping.as_ref().ok_or_else(|| {
        TemporalBackendError::new(
            "TEMPORAL_SOURCE_CLOCK_UNAVAILABLE",
            binding_id(budget),
            "source-time clock requires a resolved source mapping",
        )
    })?;
    match &mapping.time_map {
        ResolvedSourceTimeMap::Linear {
            source_range_per_repeat,
            rate,
            repeat,
            direction,
        } => linear(
            clip,
            local,
            *source_range_per_repeat,
            *rate,
            *repeat,
            *direction,
            budget,
        ),
        ResolvedSourceTimeMap::Curve { segments } => curve(local, segments, budget),
    }
}

pub(super) fn sample(clip: &ResolvedClip, local: f64) -> Option<f64> {
    let mapping = clip.source_mapping.as_ref()?;
    match &mapping.time_map {
        ResolvedSourceTimeMap::Linear {
            source_range_per_repeat,
            rate,
            repeat,
            direction,
        } => {
            let start = seconds(source_range_per_repeat.start);
            let end = seconds(source_range_per_repeat.end().ok()?);
            if local >= seconds(clip.record_range.duration) {
                return Some(match direction {
                    PlaybackDirection::Forward => end,
                    PlaybackDirection::Reverse => start,
                });
            }
            let period = seconds(clip.record_range.duration) / f64::from(*repeat);
            let within = if *repeat > 1 {
                local.rem_euclid(period)
            } else {
                local
            };
            let distance = within * rate.numerator as f64 / f64::from(rate.denominator);
            Some(match direction {
                PlaybackDirection::Forward => start + distance,
                PlaybackDirection::Reverse => end - distance,
            })
        }
        ResolvedSourceTimeMap::Curve { segments } => {
            let mut elapsed = 0.0;
            for segment in segments {
                let duration = seconds(segment.record_duration);
                if local < elapsed + duration {
                    let start = seconds(segment.source_start);
                    return Some(match segment.interpolation {
                        SourceTimeInterpolation::Hold => start,
                        SourceTimeInterpolation::Linear => {
                            let progress = (local - elapsed) / duration;
                            start + (seconds(segment.source_end) - start) * progress
                        }
                    });
                }
                elapsed += duration;
            }
            segments.last().map(|value| seconds(value.source_end))
        }
    }
}

fn linear(
    clip: &ResolvedClip,
    local: &Expression,
    range: veac_plan::canonical::TimeRange,
    rate: veac_plan::canonical::Rational,
    repeat: u32,
    direction: PlaybackDirection,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let start = number(budget, seconds(range.start))?;
    let end = number(
        budget,
        seconds(
            range
                .end()
                .map_err(|_| budget.contract("source range overflows"))?,
        ),
    )?;
    let rate = number(budget, rate.numerator as f64 / f64::from(rate.denominator))?;
    let period = number(
        budget,
        seconds(clip.record_range.duration) / f64::from(repeat),
    )?;
    let within = if repeat > 1 {
        expression::call2(budget, "mod", local, &period)?
    } else {
        local.clone()
    };
    let distance = expression::infix(budget, &within, "*", &rate)?;
    let sampled = match direction {
        PlaybackDirection::Forward => expression::infix(budget, &start, "+", &distance)?,
        PlaybackDirection::Reverse => expression::infix(budget, &end, "-", &distance)?,
    };
    let duration = number(budget, seconds(clip.record_range.duration))?;
    let at_end = expression::call2(budget, "gte", local, &duration)?;
    let final_value = match direction {
        PlaybackDirection::Forward => end,
        PlaybackDirection::Reverse => start,
    };
    expression::select(budget, &at_end, &final_value, &sampled)
}

fn curve(
    local: &Expression,
    segments: &[veac_plan::canonical::SourceTimeSegment],
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let Some(last) = segments.last() else {
        return Err(budget.contract("source-time curve is empty"));
    };
    let mut result = number(budget, seconds(last.source_end))?;
    let mut starts = Vec::with_capacity(segments.len());
    let mut elapsed = 0.0;
    for segment in segments {
        starts.push(elapsed);
        elapsed += seconds(segment.record_duration);
    }
    for (segment, start) in segments.iter().zip(starts).rev() {
        let start_value = number(budget, seconds(segment.source_start))?;
        let sampled = match segment.interpolation {
            SourceTimeInterpolation::Hold => start_value,
            SourceTimeInterpolation::Linear => {
                let origin = number(budget, start)?;
                let offset = expression::infix(budget, local, "-", &origin)?;
                let duration = number(budget, seconds(segment.record_duration))?;
                let progress = expression::infix(budget, &offset, "/", &duration)?;
                let end_value = number(budget, seconds(segment.source_end))?;
                let delta = expression::infix(budget, &end_value, "-", &start_value)?;
                let delta = expression::infix(budget, &delta, "*", &progress)?;
                expression::infix(budget, &start_value, "+", &delta)?
            }
        };
        let boundary = number(budget, start + seconds(segment.record_duration))?;
        let condition = expression::call2(budget, "lt", local, &boundary)?;
        result = expression::select(budget, &condition, &sampled, &result)?;
    }
    Ok(result)
}

fn seconds(value: veac_plan::canonical::RationalTime) -> f64 {
    value.value as f64 / f64::from(value.timescale)
}

fn binding_id<'a>(budget: &'a Budget<'_>) -> &'a veac_plan::canonical::TemporalBindingId {
    budget.binding_id()
}
