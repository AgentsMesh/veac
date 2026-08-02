use veac_plan::canonical::{Animatable, PitchPolicy};
use veac_plan::ResolvedClip;

use super::audio::AudioRenderSpec;
use super::{animation, audio_filters, audio_source::invalid, time, CodegenErrors, EmitContext};

pub(super) fn speed(
    context: &mut EmitContext<'_>,
    input: &str,
    rate: veac_plan::canonical::Rational,
    sample_rate: u32,
    pitch: PitchPolicy,
) -> String {
    if rate.numerator == i64::from(rate.denominator) {
        return input.to_owned();
    }
    let rate = rate.numerator as f64 / f64::from(rate.denominator);
    speed_factor(context, input, rate, sample_rate, pitch)
}

pub(super) fn speed_factor(
    context: &mut EmitContext<'_>,
    input: &str,
    rate: f64,
    sample_rate: u32,
    pitch: PitchPolicy,
) -> String {
    let filter = match pitch {
        PitchPolicy::FollowSpeed => format!(
            "asetrate={}*{},aresample={sample_rate}",
            sample_rate,
            time::number(rate)
        ),
        PitchPolicy::Preserve => atempo(rate),
    };
    context.graph.filter(&[input], filter, "speeda")
}

pub(super) fn repeat(context: &mut EmitContext<'_>, input: &str, count: u32) -> String {
    if count <= 1 {
        return input.to_owned();
    }
    let copies = context.graph.filter_many(
        &[input],
        format!("asplit={count}"),
        "loopcopya",
        count as usize,
    );
    let refs: Vec<_> = copies.iter().map(String::as_str).collect();
    context
        .graph
        .filter(&refs, format!("concat=n={count}:v=0:a=1"), "loopa")
}

pub(super) fn properties(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    mut label: String,
    output: &AudioRenderSpec,
) -> Result<String, CodegenErrors> {
    let properties = clip
        .audio
        .as_ref()
        .ok_or_else(|| invalid(clip, "audio properties missing"))?;
    let gain = animation::number(&properties.gain, "t");
    label = context
        .graph
        .filter(&[&label], format!("volume='{gain}':eval=frame"), "gaina");
    label = pan(context, clip, label, &properties.pan, output.channels)?;
    if properties.normalize {
        label = context
            .graph
            .filter(&[&label], "loudnorm=I=-16:LRA=11:TP=-1.5", "norma");
    }
    Ok(audio_filters::apply(context, label, &properties.processors))
}

fn atempo(mut rate: f64) -> String {
    let mut filters = Vec::new();
    while rate > 100.0 {
        filters.push("atempo=100".to_owned());
        rate /= 100.0;
    }
    while rate < 0.5 {
        filters.push("atempo=0.5".to_owned());
        rate /= 0.5;
    }
    filters.push(format!("atempo={}", time::number(rate)));
    filters.join(",")
}

fn pan(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    input: String,
    value: &Animatable<f64>,
    channels: u8,
) -> Result<String, CodegenErrors> {
    if channels != 2 {
        return match value {
            Animatable::Constant { value } if *value == 0.0 => Ok(input),
            _ => Err(invalid(clip, "pan automation requires stereo output")),
        };
    }
    let expression = animation::number(value, "t");
    let formatted = context
        .graph
        .filter(&[&input], "aformat=channel_layouts=stereo", "stereoa");
    let channels = context.graph.filter_many(
        &[&formatted],
        "channelsplit=channel_layout=stereo",
        "channel",
        2,
    );
    let left = context.graph.filter(
        &[&channels[0]],
        format!("volume='if(lte({expression}\\,0)\\,1\\,1-({expression}))':eval=frame"),
        "panleft",
    );
    let right = context.graph.filter(
        &[&channels[1]],
        format!("volume='if(gte({expression}\\,0)\\,1\\,1+({expression}))':eval=frame"),
        "panright",
    );
    Ok(context
        .graph
        .filter(&[&left, &right], "amerge=inputs=2", "pana"))
}
