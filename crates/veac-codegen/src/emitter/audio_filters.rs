use veac_plan::canonical::{
    AudioProcessor, AudioProcessorKind, Compressor, Gate, Limiter, LoudnessTarget,
};

use super::{time, EmitContext};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    mut input: String,
    processors: &[AudioProcessor],
) -> String {
    for processor in processors {
        let filter = match &processor.kind {
            AudioProcessorKind::ParametricEq { bands } => {
                for band in bands {
                    input = context.graph.filter(
                        &[&input],
                        format!(
                            "equalizer=f={}:t=q:w={}:g={}",
                            number(band.frequency_hz),
                            number(band.q),
                            number(band.gain_db)
                        ),
                        "eqa",
                    );
                }
                continue;
            }
            AudioProcessorKind::HighPass {
                frequency_hz,
                q,
                poles,
            } => pass_filter("highpass", *frequency_hz, *q, *poles),
            AudioProcessorKind::LowPass {
                frequency_hz,
                q,
                poles,
            } => pass_filter("lowpass", *frequency_hz, *q, *poles),
            AudioProcessorKind::Compressor(value) => compressor(*value),
            AudioProcessorKind::Limiter(value) => limiter(*value),
            AudioProcessorKind::Gate(value) => gate(*value),
            AudioProcessorKind::Loudness(value) => loudness(*value),
        };
        input = context.graph.filter(&[&input], filter, "processa");
    }
    input
}

fn pass_filter(name: &str, frequency: f64, q: f64, poles: u8) -> String {
    format!(
        "{name}=f={}:t=q:w={}:p={poles}",
        number(frequency),
        number(q)
    )
}

fn compressor(value: Compressor) -> String {
    format!(
        "acompressor=threshold={}:ratio={}:attack={}:release={}:knee={}:makeup={}:mix={}",
        amplitude(value.threshold_db),
        number(value.ratio),
        number(value.attack_ms),
        number(value.release_ms),
        amplitude(value.knee_db),
        amplitude(value.makeup_gain_db),
        number(value.mix)
    )
}

fn limiter(value: Limiter) -> String {
    format!(
        "alimiter=limit={}:attack={}:release={}:level=false:latency=true",
        amplitude(value.ceiling_db),
        number(value.attack_ms),
        number(value.release_ms)
    )
}

fn gate(value: Gate) -> String {
    format!(
        "agate=threshold={}:ratio={}:attack={}:release={}:range={}",
        amplitude(value.threshold_db),
        number(value.ratio),
        number(value.attack_ms),
        number(value.release_ms),
        amplitude(value.range_db)
    )
}

fn loudness(value: LoudnessTarget) -> String {
    format!(
        "loudnorm=I={}:TP={}:LRA={}:linear=true:print_format=none",
        number(value.integrated_lufs),
        number(value.true_peak_dbtp),
        number(value.loudness_range_lu)
    )
}

pub(super) fn amplitude(db: f64) -> String {
    number(10.0_f64.powf(db / 20.0))
}

fn number(value: f64) -> String {
    time::number(value)
}
