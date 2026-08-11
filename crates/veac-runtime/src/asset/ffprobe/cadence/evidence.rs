use serde::Deserialize;
use veac_ir::{Rational, VideoCadence};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Output {
    #[serde(default)]
    packets: Vec<Packet>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    pts: Option<i64>,
    duration: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::asset::ffprobe) struct Evidence {
    pub(super) cadence: VideoCadence,
    pub(super) frame_rate: Option<Rational>,
}

pub(super) fn classify(bytes: &[u8], time_base: Rational, candidate: Option<Rational>) -> Evidence {
    let Some(mut packets) = parse(bytes) else {
        return unknown();
    };
    if packets.len() < 2 {
        return unknown();
    }
    packets.sort_unstable_by_key(|packet| packet.0);
    if let Some(rate) =
        candidate.filter(|rate| rate.is_positive() && super::rate_within_limit(*rate))
    {
        match proves_candidate(&packets, time_base, rate) {
            Some(true) => return constant(rate),
            None => return unknown(),
            Some(false) => {}
        }
    }
    exact_rate(&packets, time_base)
}

fn exact_rate(packets: &[(i64, i64)], time_base: Rational) -> Evidence {
    let Some(delta) = packets[1].0.checked_sub(packets[0].0) else {
        return unknown();
    };
    if delta <= 0 {
        return variable();
    }
    let equal_pts = packets.windows(2).all(|pair| {
        pair[1]
            .0
            .checked_sub(pair[0].0)
            .is_some_and(|value| value == delta)
    });
    let equal_durations = packets.iter().all(|packet| packet.1 == delta);
    if !equal_pts || !equal_durations {
        return variable();
    }
    let Some(denominator) = delta
        .checked_mul(time_base.numerator)
        .and_then(|value| u32::try_from(value).ok())
    else {
        return unknown();
    };
    let Ok(rate) = Rational::new(i64::from(time_base.denominator), denominator) else {
        return unknown();
    };
    if !super::rate_within_limit(rate) {
        return unknown();
    }
    constant(rate)
}

fn proves_candidate(packets: &[(i64, i64)], time_base: Rational, rate: Rational) -> Option<bool> {
    let ticks_numerator =
        i128::from(rate.denominator).checked_mul(i128::from(time_base.denominator))?;
    let ticks_denominator =
        i128::from(rate.numerator).checked_mul(i128::from(time_base.numerator))?;
    if ticks_numerator <= 0 || ticks_denominator <= 0 {
        return Some(false);
    }
    let first = packets.first()?.0;
    for (index, (pts, duration)) in packets.iter().enumerate() {
        if *duration <= 0 || index > 0 && packets[index - 1].0 >= *pts {
            return Some(false);
        }
        let elapsed = i128::from(*pts).checked_sub(i128::from(first))?;
        let expected = i128::try_from(index).ok()?.checked_mul(ticks_numerator)?;
        let phase = elapsed
            .checked_mul(ticks_denominator)?
            .checked_sub(expected)?
            .checked_abs()?;
        let duration_error = i128::from(*duration)
            .checked_mul(ticks_denominator)?
            .checked_sub(ticks_numerator)?
            .checked_abs()?;
        if phase >= ticks_denominator || duration_error >= ticks_denominator {
            return Some(false);
        }
    }
    Some(true)
}

fn parse(bytes: &[u8]) -> Option<Vec<(i64, i64)>> {
    let output: Output = serde_json::from_slice(bytes).ok()?;
    output
        .packets
        .into_iter()
        .map(|packet| Some((packet.pts?, packet.duration?)))
        .collect()
}

fn unknown() -> Evidence {
    Evidence {
        cadence: VideoCadence::Unknown,
        frame_rate: None,
    }
}

fn variable() -> Evidence {
    Evidence {
        cadence: VideoCadence::Variable,
        frame_rate: None,
    }
}

fn constant(rate: Rational) -> Evidence {
    Evidence {
        cadence: VideoCadence::Constant,
        frame_rate: Some(rate),
    }
}

#[cfg(test)]
#[path = "evidence/tests.rs"]
mod tests;
