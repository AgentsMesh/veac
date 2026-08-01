use std::cmp::Ordering;

use veac_artifact::SourceClock;
use veac_plan::canonical::{RationalTime, SourceOutOfRangePolicy};
use veac_plan::{ResolvedClip, ResolvedSourceMapping};

use super::audio::AudioRenderSpec;
use super::error::{diagnostic, CodegenErrorKind};
use super::{CodegenErrors, EmitContext};

mod audio;
mod extent;
mod math;
mod video;

#[cfg(test)]
#[path = "../unit_tests/source_boundary_internal_tests.rs"]
mod internal_tests;

#[derive(Clone, Copy)]
pub(super) struct Padding {
    pub(super) before: RationalTime,
    pub(super) after: RationalTime,
}

pub(super) fn video(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    raw: &str,
    mapping: &ResolvedSourceMapping,
    clock: SourceClock,
    duration: Option<RationalTime>,
) -> Result<(String, SourceClock), CodegenErrors> {
    let Some(padding) = required(clip, mapping, clock, duration, true)? else {
        return Ok((raw.to_owned(), clock));
    };
    video::pad(context, raw, clip, clock, padding)
}

pub(super) fn audio(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    raw: &str,
    mapping: &ResolvedSourceMapping,
    clock: SourceClock,
    duration: Option<RationalTime>,
    output: &AudioRenderSpec,
) -> Result<(String, SourceClock), CodegenErrors> {
    let Some(padding) = required(clip, mapping, clock, duration, false)? else {
        return Ok((raw.to_owned(), clock));
    };
    audio::pad(context, raw, clip, clock, padding, output)
}

fn required(
    clip: &ResolvedClip,
    mapping: &ResolvedSourceMapping,
    clock: SourceClock,
    duration: Option<RationalTime>,
    include_points: bool,
) -> Result<Option<Padding>, CodegenErrors> {
    let Some(extent) = extent::mapping(&mapping.time_map, include_points) else {
        return Ok(None);
    };
    let Some(duration) = duration else {
        return Err(invalid(
            clip,
            "source boundary policy requires a known stream duration",
        ));
    };
    let zero = RationalTime::zero(duration.timescale)
        .map_err(|_| invalid(clip, "source duration timebase is invalid"))?;
    let before = if extent.minimum.partial_cmp(&zero) == Some(Ordering::Less) {
        math::difference(zero, extent.minimum)
    } else {
        Some(zero)
    };
    let after = match extent.maximum.partial_cmp(&duration) {
        Some(Ordering::Greater) => math::difference(extent.maximum, duration),
        Some(Ordering::Equal) if extent.maximum_is_point => math::tick_after(extent.maximum),
        Some(_) => Some(zero),
        None => None,
    };
    let padding = Padding {
        before: before
            .ok_or_else(|| invalid(clip, "source boundary start is not representable"))?,
        after: after.ok_or_else(|| invalid(clip, "source boundary end is not representable"))?,
    };
    policy(clip, mapping.out_of_range, padding)?;
    if padding.before.value == 0 && padding.after.value == 0 {
        return Ok(None);
    }
    binding_edges(clip, clock, duration, padding)?;
    Ok(Some(padding))
}

fn policy(
    clip: &ResolvedClip,
    policy: SourceOutOfRangePolicy,
    padding: Padding,
) -> Result<(), CodegenErrors> {
    if padding.before.value > 0 && !policy.allows_before() {
        return Err(invalid(
            clip,
            "source mapping crosses the first-frame boundary",
        ));
    }
    if padding.after.value > 0 && !policy.allows_after() {
        return Err(invalid(
            clip,
            "source mapping crosses the last-frame boundary",
        ));
    }
    Ok(())
}

fn binding_edges(
    clip: &ResolvedClip,
    clock: SourceClock,
    duration: RationalTime,
    padding: Padding,
) -> Result<(), CodegenErrors> {
    let range = clock.logical_range().ok_or_else(|| {
        invalid(
            clip,
            "source boundary policy requires a bounded binding clock",
        )
    })?;
    let zero = RationalTime::zero(range.start.timescale)
        .map_err(|_| invalid(clip, "binding clock is invalid"))?;
    if padding.before.value > 0 && range.start != zero {
        return Err(invalid(
            clip,
            "binding does not cover the source first frame",
        ));
    }
    let end = range
        .end()
        .map_err(|_| invalid(clip, "binding clock end is invalid"))?;
    if padding.after.value > 0 && end.partial_cmp(&duration) != Some(Ordering::Equal) {
        return Err(invalid(
            clip,
            "binding does not cover the source last frame",
        ));
    }
    Ok(())
}

fn invalid(clip: &ResolvedClip, message: &str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidPlan,
        "SOURCE_BOUNDARY_POLICY",
        Some(clip.id.to_string()),
        message,
    ))
}
