use veac_plan::canonical::TimeRange;
use veac_plan::{ResolvedClip, ResolvedSidechain};

use super::super::{time, CodegenErrors, EmitContext};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    input: &str,
    sidechain: &str,
    value: &ResolvedSidechain,
    range: TimeRange,
) -> Result<String, CodegenErrors> {
    let end = range
        .end()
        .map_err(|_| super::unsupported(clip, "invalid partial sidechain range"))?;
    let has_before = range.start.value > 0;
    let has_after = end < clip.record_range.duration;
    let count = 1 + usize::from(has_before) + usize::from(has_after);
    let mut inputs = context
        .graph
        .filter_many(&[input], format!("asplit={count}"), "sidechainrange", count)
        .into_iter();
    let mut segments = Vec::with_capacity(count);
    if has_before {
        let before = inputs.next().expect("split count includes prefix");
        segments.push(trim(context, &before, "0", &time::seconds(range.start)));
    }
    let middle = inputs.next().expect("split count includes active range");
    let start = time::seconds(range.start);
    let finish = time::seconds(end);
    let duration = time::seconds(range.duration);
    let middle = trim(context, &middle, &start, &finish);
    let middle = pad(context, &middle);
    let control = trim(context, sidechain, &start, &finish);
    let control = pad(context, &control);
    let compressed = super::compress(context, &middle, &control, value);
    segments.push(trim_duration(context, &compressed, &duration));
    if has_after {
        let after = inputs.next().expect("split count includes suffix");
        segments.push(trim(
            context,
            &after,
            &finish,
            &time::seconds(clip.record_range.duration),
        ));
    }
    let labels: Vec<_> = segments.iter().map(String::as_str).collect();
    Ok(context.graph.filter(
        &labels,
        format!("concat=n={count}:v=0:a=1"),
        "sidechainrange",
    ))
}

fn trim(context: &mut EmitContext<'_>, input: &str, start: &str, end: &str) -> String {
    context.graph.filter(
        &[input],
        format!("atrim=start={start}:end={end},asetpts=PTS-STARTPTS"),
        "sidechainrange",
    )
}

fn pad(context: &mut EmitContext<'_>, input: &str) -> String {
    context.graph.filter(&[input], "apad", "sidechainrange")
}

fn trim_duration(context: &mut EmitContext<'_>, input: &str, duration: &str) -> String {
    context.graph.filter(
        &[input],
        format!("atrim=duration={duration},asetpts=PTS-STARTPTS"),
        "sidechainrange",
    )
}
