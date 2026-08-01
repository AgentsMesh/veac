use veac_plan::canonical::{
    AdaptivePackage, Animatable, DeliverableKind, PitchPolicy, SourceTimeInterpolation,
};
use veac_plan::{EffectiveAudioProperties, ResolvedRenderPlan, ResolvedSourceTimeMap};

use crate::emitter::input::usage::{visit_audio, AudioSelection};

use super::super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    for deliverable in &plan.output.deliverables {
        let (sample_rate, channels, selection) = match &deliverable.kind {
            DeliverableKind::Video(settings) => {
                let Some(output) = settings.audio.as_ref() else {
                    continue;
                };
                (output.sample_rate, output.channels, AudioSelection::Master)
            }
            DeliverableKind::AudioStem(settings) => (
                settings.audio.sample_rate,
                settings.audio.channels,
                AudioSelection::from(&settings.source),
            ),
            DeliverableKind::AudioFile(settings) => {
                let veac_plan::canonical::AudioFileEncoding::Mp3(encoding) = &settings.encoding;
                (
                    encoding.sample_rate_hz,
                    encoding.channel_layout.count(),
                    AudioSelection::from(&settings.source),
                )
            }
            DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(settings)) => {
                let Some(output) = settings.audio.as_ref() else {
                    continue;
                };
                let veac_plan::canonical::HlsAudioEncoding::Aac(encoding) = &output.encoding;
                (
                    encoding.sample_rate_hz,
                    encoding.channel_layout.count(),
                    AudioSelection::from(&output.source),
                )
            }
            _ => continue,
        };
        visit_audio(plan, selection, |clip| {
            validate_clip(check, clip, sample_rate, channels)
        });
    }
}

fn validate_clip(
    check: &mut Check,
    clip: &veac_plan::ResolvedClip,
    sample_rate: u32,
    channels: u8,
) {
    let audio = clip
        .audio
        .as_ref()
        .expect("audio usage only visits audible clips");
    output_contract(check, clip, audio, sample_rate, channels);
    let follows_speed = audio.pitch_policy == PitchPolicy::FollowSpeed;
    let valid = !follows_speed
        || clip
            .source_mapping
            .as_ref()
            .is_none_or(|mapping| mapping_valid(&mapping.time_map, sample_rate));
    if !valid {
        check.push(
            "PLAN_AUDIO_RATE_INVALID",
            Some(clip.id.to_string()),
            "follow-speed pitch requires an executable FFmpeg asetrate value",
        );
    }
}

fn output_contract(
    check: &mut Check,
    clip: &veac_plan::ResolvedClip,
    audio: &EffectiveAudioProperties,
    sample_rate: u32,
    channels: u8,
) {
    let numerator = i128::from(clip.record_range.start.value) * i128::from(sample_rate);
    if clip.record_range.start.timescale == 0
        || numerator < 0
        || numerator % i128::from(clip.record_range.start.timescale) != 0
    {
        check.push(
            "PLAN_AUDIO_SAMPLE_ALIGNMENT_INVALID",
            Some(clip.id.to_string()),
            "record start is not aligned to an output audio sample",
        );
    }
    if pan_nonzero(&audio.pan) && channels != 2 {
        check.push(
            "PLAN_AUDIO_CHANNEL_LAYOUT_INVALID",
            Some(clip.id.to_string()),
            "pan automation requires stereo output",
        );
    }
}

fn pan_nonzero(value: &Animatable<f64>) -> bool {
    match value {
        Animatable::Constant { value } => *value != 0.0,
        Animatable::Keyframes { keyframes } => keyframes.iter().any(|key| key.value != 0.0),
    }
}

fn mapping_valid(value: &ResolvedSourceTimeMap, sample_rate: u32) -> bool {
    match value {
        ResolvedSourceTimeMap::Linear { rate, .. } => {
            scaled_valid(rate.numerator, i64::from(rate.denominator), sample_rate)
        }
        ResolvedSourceTimeMap::Curve { segments } => segments.iter().all(|segment| {
            segment.interpolation == SourceTimeInterpolation::Hold
                || segment
                    .source_end
                    .value
                    .checked_sub(segment.source_start.value)
                    .and_then(i64::checked_abs)
                    .is_some_and(|span| {
                        scaled_valid(span, segment.record_duration.value, sample_rate)
                    })
        }),
    }
}

fn scaled_valid(numerator: i64, denominator: i64, sample_rate: u32) -> bool {
    if numerator <= 0 || denominator <= 0 {
        return false;
    }
    let scaled = i128::from(sample_rate) * i128::from(numerator);
    let denominator = i128::from(denominator);
    scaled >= denominator && scaled <= i128::from(i32::MAX) * denominator
}
