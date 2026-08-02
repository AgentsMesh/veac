use std::collections::BTreeSet;

use veac_plan::canonical::{HashAlgorithm, MaterialId, MaterialKind, RationalTime};
use veac_plan::{PlanInputId, ResolvedInput, ResolvedInputKind, ResolvedRenderPlan};

use super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    let mut ids = BTreeSet::new();
    if !plan.inputs.windows(2).all(|pair| pair[0].id < pair[1].id) {
        check.push(
            "PLAN_INPUT_ORDER_INVALID",
            None,
            "resolved inputs must be sorted and unique",
        );
    }
    for input in &plan.inputs {
        if PlanInputId::new(input.id.as_str()).is_err()
            || !ids.insert(input.id.to_string())
            || input
                .material_id
                .as_ref()
                .is_none_or(|id| MaterialId::new(id.as_str()).is_err())
            || input.canonical_uri.is_empty()
            || input.canonical_uri.chars().any(char::is_control)
            || !identity_valid(
                input.observed_identity.algorithm,
                &input.observed_identity.digest,
            )
            || !kind_valid(input)
        {
            check.push(
                "PLAN_INPUT_FACTS_INVALID",
                Some(input.id.to_string()),
                "input identity, kind, probe, or selected stream facts are invalid",
            );
        }
    }
}

fn kind_valid(input: &ResolvedInput) -> bool {
    match &input.kind {
        ResolvedInputKind::Media { material_kind } => {
            input.probe.as_ref().is_some_and(probe_valid)
                && input.video.as_ref().is_none_or(video_valid)
                && input.audio.as_ref().is_none_or(audio_valid)
                && input
                    .video
                    .as_ref()
                    .zip(input.audio.as_ref())
                    .is_none_or(|(video, audio)| {
                        video.selection.global_index != audio.selection.global_index
                    })
                && match *material_kind {
                    MaterialKind::Video => input.video.is_some(),
                    MaterialKind::Image => input.video.is_some() && input.audio.is_none(),
                    MaterialKind::Audio => input.video.is_none() && input.audio.is_some(),
                    _ => false,
                }
        }
        ResolvedInputKind::Font {
            family,
            postscript_name,
            ..
        } => {
            input.probe.is_none()
                && input.video.is_none()
                && input.audio.is_none()
                && family.as_ref().is_none_or(|value| !value.trim().is_empty())
                && postscript_name
                    .as_ref()
                    .is_none_or(|value| !value.trim().is_empty())
        }
        ResolvedInputKind::Resource { material_kind } => {
            matches!(material_kind, MaterialKind::Lut1d | MaterialKind::Lut3d)
                && input.probe.is_none()
                && input.video.is_none()
                && input.audio.is_none()
        }
    }
}

fn probe_valid(value: &veac_plan::ResolvedMediaProbe) -> bool {
    value.schema_version == veac_plan::canonical::MEDIA_PROBE_SCHEMA_VERSION
        && !value.engine.trim().is_empty()
        && !value.selection_policy.trim().is_empty()
        && veac_plan::canonical::container_format_valid(&value.container_format)
        && value
            .container_duration
            .is_none_or(|time| intrinsic(time, true))
}

fn video_valid(value: &veac_plan::ResolvedVideoStream) -> bool {
    !value.codec.trim().is_empty()
        && value.start_time.is_none_or(|time| intrinsic(time, false))
        && value.duration.is_none_or(|time| intrinsic(time, true))
        && veac_plan::canonical::input_video_geometry_valid(&value.info)
        && !value.disposition.attached_picture
        && !value.disposition.timed_thumbnail
}

fn audio_valid(value: &veac_plan::ResolvedAudioStream) -> bool {
    !value.codec.trim().is_empty()
        && value.start_time.is_none_or(|time| intrinsic(time, false))
        && value.duration.is_none_or(|time| intrinsic(time, true))
        && veac_plan::canonical::input_audio_stream_valid(&value.info)
}

fn intrinsic(value: RationalTime, positive: bool) -> bool {
    value.is_valid()
        && if positive {
            value.value > 0
        } else {
            value.value >= 0
        }
}

fn identity_valid(algorithm: HashAlgorithm, digest: &str) -> bool {
    algorithm == HashAlgorithm::Sha256
        && digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
