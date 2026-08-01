use veac_plan::canonical::{RationalTime, TimeRange};

use super::{time, CodegenErrors, EmitContext};

pub(super) fn map<F>(
    context: &mut EmitContext<'_>,
    input: String,
    window: TimeRange,
    active: TimeRange,
    clock_origin: RationalTime,
    prefix: &str,
    process: F,
) -> Result<String, CodegenErrors>
where
    F: FnOnce(&mut EmitContext<'_>, String) -> Result<String, CodegenErrors>,
{
    let start = offset(window.start, active.start);
    let end = start
        .checked_add(active.duration)
        .expect("preflight validates ranges");
    let before = start.value > 0;
    let after = end < window.duration;
    let count = 1 + usize::from(before) + usize::from(after);
    let mut branches = if count == 1 {
        vec![input]
    } else {
        context
            .graph
            .filter_many(&[&input], format!("split={count}"), prefix, count)
    };
    let mut pieces = Vec::with_capacity(count);
    if before {
        let branch = branches.remove(0);
        pieces.push(trim(context, &branch, zero(start.timescale), start, prefix));
    }
    let branch = branches.remove(0);
    let clock = offset(clock_origin, active.start);
    let segment = trim_with_clock(context, &branch, start, active.duration, clock, prefix);
    let processed = process(context, segment)?;
    pieces.push(
        context
            .graph
            .filter(&[&processed], "setpts=PTS-STARTPTS", prefix),
    );
    if after {
        let branch = branches.remove(0);
        pieces.push(trim(context, &branch, end, window.duration, prefix));
    }
    if pieces.len() == 1 {
        Ok(pieces.remove(0))
    } else {
        let labels: Vec<_> = pieces.iter().map(String::as_str).collect();
        Ok(context.graph.filter(
            &labels,
            format!("concat=n={}:v=1:a=0", pieces.len()),
            prefix,
        ))
    }
}

pub(super) fn offset(from: RationalTime, to: RationalTime) -> RationalTime {
    assert_eq!(from.timescale, to.timescale, "preflight aligns timebase");
    RationalTime {
        value: to.value - from.value,
        timescale: from.timescale,
    }
}

pub(super) fn intersect(left: TimeRange, right: TimeRange) -> Option<TimeRange> {
    let start = if left.start > right.start {
        left.start
    } else {
        right.start
    };
    let left_end = left.end().ok()?;
    let right_end = right.end().ok()?;
    let end = if left_end < right_end {
        left_end
    } else {
        right_end
    };
    (start < end).then(|| TimeRange {
        start,
        duration: offset(start, end),
    })
}

fn trim(
    context: &mut EmitContext<'_>,
    input: &str,
    start: RationalTime,
    end: RationalTime,
    prefix: &str,
) -> String {
    context.graph.filter(
        &[input],
        format!(
            "trim=start={}:end={},setpts=PTS-STARTPTS",
            time::seconds(start),
            time::seconds(end)
        ),
        prefix,
    )
}

fn trim_with_clock(
    context: &mut EmitContext<'_>,
    input: &str,
    start: RationalTime,
    duration: RationalTime,
    clock: RationalTime,
    prefix: &str,
) -> String {
    context.graph.filter(
        &[input],
        format!(
            "trim=start={}:duration={},setpts=PTS-STARTPTS+{}/TB",
            time::seconds(start),
            time::seconds(duration),
            time::seconds(clock)
        ),
        prefix,
    )
}

fn zero(timescale: u32) -> RationalTime {
    RationalTime {
        value: 0,
        timescale,
    }
}
