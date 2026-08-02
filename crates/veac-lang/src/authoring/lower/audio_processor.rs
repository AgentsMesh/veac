use crate::authoring::AudioProcessorDecl;
use veac_ir::{AudioProcessor, Compressor, Gate, Limiter, LoudnessTarget, ParametricEqBand};

use super::context::Context;
use super::value;

pub(super) fn lower(ctx: &mut Context, value: &AudioProcessorDecl) -> Option<AudioProcessor> {
    Some(match value {
        AudioProcessorDecl::ParametricEq { bands, .. } => AudioProcessor::ParametricEq {
            bands: bands
                .iter()
                .map(|band| {
                    Some(ParametricEqBand {
                        frequency_hz: value::scalar(ctx, &band.frequency, "hz")?,
                        gain_db: value::scalar(ctx, &band.gain, "db")?,
                        q: value::unitless(ctx, &band.q)?,
                    })
                })
                .collect::<Option<Vec<_>>>()?,
        },
        AudioProcessorDecl::HighPass(value) => AudioProcessor::HighPass {
            frequency_hz: scalar(ctx, &value.frequency, "hz")?,
            q: unitless(ctx, &value.q)?,
            poles: poles(ctx, &value.poles)?,
        },
        AudioProcessorDecl::LowPass(value) => AudioProcessor::LowPass {
            frequency_hz: scalar(ctx, &value.frequency, "hz")?,
            q: unitless(ctx, &value.q)?,
            poles: poles(ctx, &value.poles)?,
        },
        AudioProcessorDecl::Compressor(value) => AudioProcessor::Compressor(Compressor {
            threshold_db: scalar(ctx, &value.threshold, "db")?,
            ratio: unitless(ctx, &value.ratio)?,
            attack_ms: scalar(ctx, &value.attack, "ms")?,
            release_ms: scalar(ctx, &value.release, "ms")?,
            knee_db: scalar(ctx, &value.knee, "db")?,
            makeup_gain_db: scalar(ctx, &value.makeup_gain, "db")?,
            mix: value::scale(ctx, &value.mix)?,
        }),
        AudioProcessorDecl::Limiter(value) => AudioProcessor::Limiter(Limiter {
            ceiling_db: scalar(ctx, &value.ceiling, "db")?,
            attack_ms: scalar(ctx, &value.attack, "ms")?,
            release_ms: scalar(ctx, &value.release, "ms")?,
        }),
        AudioProcessorDecl::Gate(value) => AudioProcessor::Gate(Gate {
            threshold_db: scalar(ctx, &value.threshold, "db")?,
            ratio: unitless(ctx, &value.ratio)?,
            attack_ms: scalar(ctx, &value.attack, "ms")?,
            release_ms: scalar(ctx, &value.release, "ms")?,
            range_db: scalar(ctx, &value.range, "db")?,
        }),
        AudioProcessorDecl::Loudness(value) => AudioProcessor::Loudness(LoudnessTarget {
            integrated_lufs: scalar(ctx, &value.integrated, "lufs")?,
            true_peak_dbtp: scalar(ctx, &value.true_peak, "dbtp")?,
            loudness_range_lu: scalar(ctx, &value.range, "lu")?,
        }),
    })
}

fn poles(ctx: &mut Context, value: &crate::authoring::NumberLiteral) -> Option<u8> {
    let parsed = super::value::integer_u32(ctx, value, "filter poles")?;
    u8::try_from(parsed).ok().or_else(|| {
        ctx.error(
            "AUTHORING_LOWER_INTEGER_RANGE",
            "filter poles must fit in u8",
            value.span,
        );
        None
    })
}

fn scalar(ctx: &mut Context, value: &crate::authoring::NumberLiteral, unit: &str) -> Option<f64> {
    super::value::scalar(ctx, value, unit)
}

fn unitless(ctx: &mut Context, value: &crate::authoring::NumberLiteral) -> Option<f64> {
    super::value::unitless(ctx, value)
}
