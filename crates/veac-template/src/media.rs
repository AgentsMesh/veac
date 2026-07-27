use veac_ir::{
    FillMode, FrameSynthesisPolicy, Material, MaterialKind, PlaybackDirection, Rational,
    RationalTime, Rect, SourceMapping, SourceOutOfRangePolicy, SourceTimeMap, VideoStreamInfo,
};

use crate::{geometry, inventory::MediaSlot, timing, TemplateError, TemplateErrorKind};

pub(crate) struct MediaPlan {
    pub mapping: SourceMapping,
    pub crop: Rect,
}

pub(crate) fn plan(
    slot: &MediaSlot,
    material: &Material,
    timebase: u32,
) -> Result<MediaPlan, TemplateError> {
    if material.id != slot.material_id {
        return Err(TemplateError::clip(
            TemplateErrorKind::MaterialIdMismatch,
            &slot.clip_id,
            "replacement material ID must match the placeholder material ID",
        ));
    }
    if !slot.constraint.kind.accepts(material.kind) {
        return Err(TemplateError::clip(
            TemplateErrorKind::KindMismatch,
            &slot.clip_id,
            "replacement material kind is not accepted by the slot",
        ));
    }
    let (info, duration) = probe_facts(slot, material)?;
    let mapping = mapping(slot, material.kind, duration, timebase)?;
    Ok(MediaPlan {
        mapping,
        crop: geometry::crop(info, slot.frame, slot.canvas),
    })
}

fn probe_facts(
    slot: &MediaSlot,
    material: &Material,
) -> Result<(VideoStreamInfo, Option<RationalTime>), TemplateError> {
    let probe = material.probe.as_ref().ok_or_else(|| {
        TemplateError::clip(
            TemplateErrorKind::MissingProbe,
            &slot.clip_id,
            "replacement material has no probe snapshot",
        )
    })?;
    let identity = material.identity.as_ref().ok_or_else(|| {
        TemplateError::clip(
            TemplateErrorKind::MissingIdentity,
            &slot.clip_id,
            "replacement material has no SHA-256 content identity",
        )
    })?;
    if identity.algorithm != veac_ir::HashAlgorithm::Sha256 || identity != &probe.observed_identity
    {
        return Err(TemplateError::clip(
            TemplateErrorKind::ProbeIdentityMismatch,
            &slot.clip_id,
            "replacement probe facts are not bound to its SHA-256 content identity",
        ));
    }
    let selection = probe
        .selected_video_stream
        .ok_or_else(|| missing_facts(slot))?;
    let stream = probe
        .streams
        .iter()
        .find(|stream| {
            stream.global_index == selection.global_index
                && stream.type_index == selection.type_index
                && stream.media_type == veac_ir::ProbedStreamType::Video
                && !stream.disposition.attached_picture
                && !stream.disposition.timed_thumbnail
        })
        .ok_or_else(|| missing_facts(slot))?;
    let info = match stream.video.as_ref() {
        Some(info)
            if info.width > 0 && info.height > 0 && info.sample_aspect_ratio.is_positive() =>
        {
            info.clone()
        }
        _ => return Err(missing_facts(slot)),
    };
    if material.kind == MaterialKind::Video && stream.duration.is_none() {
        return Err(TemplateError::clip(
            TemplateErrorKind::MissingDuration,
            &slot.clip_id,
            "replacement video selected stream has no duration",
        ));
    }
    Ok((info, stream.duration))
}

fn mapping(
    slot: &MediaSlot,
    kind: MaterialKind,
    duration: Option<RationalTime>,
    timebase: u32,
) -> Result<SourceMapping, TemplateError> {
    let zero = RationalTime {
        value: 0,
        timescale: timebase,
    };
    if kind == MaterialKind::Image {
        return Ok(SourceMapping::linear(zero, one()));
    }
    let duration = match duration {
        Some(duration) => duration,
        None => return Err(missing_facts(slot)),
    };
    let natural = timing::convert_exact(duration, timebase)?;
    if natural.value <= 0 {
        return Err(missing_facts(slot));
    }
    require_duration(slot, natural)?;
    let (source_start, rate) = match slot.constraint.fill {
        FillMode::FitDuration => (zero, timing::ratio(natural.value, slot.duration.value)?),
        FillMode::TakeHead => (zero, one()),
        FillMode::TakeCenter => (timing::centered_start(natural, slot.duration)?, one()),
    };
    Ok(SourceMapping {
        time_map: SourceTimeMap::Linear {
            source_start,
            rate,
            repeat: 1,
            direction: PlaybackDirection::Forward,
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::Strict,
    })
}

fn require_duration(slot: &MediaSlot, natural: RationalTime) -> Result<(), TemplateError> {
    let fallback = match slot.constraint.fill {
        FillMode::FitDuration => 0,
        FillMode::TakeHead | FillMode::TakeCenter => slot.duration.value,
    };
    let minimum = slot
        .constraint
        .min_source_duration
        .map_or(fallback, |time| time.value.max(fallback));
    if natural.value < minimum {
        return Err(TemplateError::clip(
            TemplateErrorKind::MediaTooShort,
            &slot.clip_id,
            "replacement video is shorter than the slot requirement",
        ));
    }
    Ok(())
}

fn one() -> Rational {
    Rational {
        numerator: 1,
        denominator: 1,
    }
}

fn missing_facts(slot: &MediaSlot) -> TemplateError {
    TemplateError::clip(
        TemplateErrorKind::MissingProbeFacts,
        &slot.clip_id,
        "replacement material lacks selected intrinsic video facts",
    )
}
