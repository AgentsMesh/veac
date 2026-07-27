use std::collections::BTreeSet;

use veac_ir::{
    Animatable, AudioFadeCurve, AudioProcessor, Interpolation, PitchPolicy, PlaybackDirection,
    ProjectEnvelope, Rational, RelationKind, SequenceId, SourceMapping, SourceOutOfRangePolicy,
    SourceTimeMap, TrackRouting,
};

use crate::support::{assert_preview_evidence, clips, entry_sequence, lower_example};

#[test]
fn preview_timing_and_audio_rows_have_typed_evidence_in_their_target() {
    assert_preview_evidence("timing-audio.json", evidence);
}

#[test]
fn non_entry_audio_is_not_preview_evidence() {
    let mut envelope = lower_example("audio-processing/main.veac");
    let entry_id = envelope.project.entry_sequence_id.clone();
    let mut hidden = {
        let entry = envelope
            .project
            .sequences
            .iter_mut()
            .find(|sequence| sequence.id == entry_id)
            .unwrap();
        let hidden = entry.clone();
        entry.tracks.clear();
        hidden
    };
    let hidden_id = SequenceId::new("seq_hidden_audio").unwrap();
    hidden.id = hidden_id.clone();
    for relation in &mut envelope.project.relations {
        if relation.sequence_id == entry_id {
            relation.sequence_id = hidden_id.clone();
        }
    }
    envelope.project.sequences.push(hidden);
    let actual = evidence(&envelope);
    assert!(!actual.contains("audio.normalize"));
    assert!(!actual.contains("audio.routing"));
    assert!(!actual.contains("audio.sidechain-ducking"));
}

fn evidence(envelope: &ProjectEnvelope) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for clip in clips(envelope) {
        if let Some(mapping) = &clip.source_mapping {
            mapping_evidence(mapping, &mut found);
        }
        if let Some(audio) = &clip.audio {
            audio_evidence(audio, &mut found);
        }
        if let Some(visual) = &clip.visual {
            interpolation_evidence(&visual.transform.position, &mut found);
        }
    }
    let entry = entry_sequence(envelope);
    if entry
        .tracks
        .iter()
        .any(|track| matches!(track.routing, TrackRouting::AudioBus { .. }))
    {
        found.insert("audio.routing".to_owned());
    }
    if envelope
        .project
        .relations
        .iter()
        .filter(|relation| relation.sequence_id == entry.id)
        .any(|relation| matches!(relation.kind, RelationKind::Sidechain { .. }))
    {
        found.insert("audio.sidechain-ducking".to_owned());
    }
    found
}

fn mapping_evidence(mapping: &SourceMapping, found: &mut BTreeSet<String>) {
    let boundary = match mapping.out_of_range {
        SourceOutOfRangePolicy::Strict => "source-time.out-of-range.reject",
        SourceOutOfRangePolicy::HoldFirst => "source-time.out-of-range.hold-first",
        SourceOutOfRangePolicy::HoldLast => "source-time.out-of-range.hold-last",
        SourceOutOfRangePolicy::HoldBoth => "source-time.out-of-range.hold-both",
    };
    found.insert(boundary.to_owned());
    match &mapping.time_map {
        SourceTimeMap::Linear {
            rate, direction, ..
        } if *rate == Rational::new(1, 1).unwrap() && *direction == PlaybackDirection::Forward => {
            found.insert("source-time.identity".to_owned());
        }
        SourceTimeMap::Linear { .. } => {
            found.insert("source-time.constant-speed".to_owned());
        }
        SourceTimeMap::Curve { .. } => {
            found.insert("source-time.time-remap".to_owned());
        }
    }
}

fn audio_evidence(audio: &veac_ir::AudioProperties, found: &mut BTreeSet<String>) {
    if !constant_eq(&audio.gain, 1.0) || !constant_eq(&audio.pan, 0.0) {
        found.insert("audio.volume-pan".to_owned());
    }
    if audio.crossfade.is_some() {
        found.insert("audio.fade".to_owned());
    }
    let pitch = match audio.pitch_policy {
        PitchPolicy::Preserve => "audio.pitch-policy-preserve",
        PitchPolicy::FollowSpeed => "audio.pitch-policy-follow-speed",
    };
    found.insert(pitch.to_owned());
    if let Some(crossfade) = &audio.crossfade {
        let curve = match crossfade.curve {
            AudioFadeCurve::Linear => "audio.fade-curve-linear",
            AudioFadeCurve::Exponential => "audio.fade-curve-exponential",
            AudioFadeCurve::EqualPower => "audio.fade-curve-equal-power",
        };
        found.insert(curve.to_owned());
    }
    if matches!(audio.gain, Animatable::Keyframes { .. })
        || matches!(audio.pan, Animatable::Keyframes { .. })
    {
        found.insert("audio.automation".to_owned());
    }
    if audio.normalize
        || audio
            .processors
            .iter()
            .any(|value| matches!(value, AudioProcessor::Loudness(_)))
    {
        found.insert("audio.normalize".to_owned());
    }
    for processor in &audio.processors {
        let id = match processor {
            AudioProcessor::ParametricEq { .. } => Some("audio.equalizer"),
            AudioProcessor::Compressor(_) => Some("audio.compressor"),
            AudioProcessor::Limiter(_) => Some("audio.limiter"),
            AudioProcessor::Gate(_) => Some("audio.noise-gate"),
            _ => None,
        };
        found.extend(id.map(str::to_owned));
    }
}

fn interpolation_evidence<T>(value: &Animatable<T>, found: &mut BTreeSet<String>) {
    let Animatable::Keyframes { keyframes } = value else {
        return;
    };
    for keyframe in keyframes {
        let id = match keyframe.interpolation {
            Interpolation::Hold => "animation.interpolation-hold",
            Interpolation::Linear => "animation.interpolation-linear",
            Interpolation::EaseIn => "animation.interpolation-ease-in",
            Interpolation::EaseOut => "animation.interpolation-ease-out",
            Interpolation::EaseInOut => "animation.interpolation-ease-in-out",
            Interpolation::CubicBezier { .. } => "animation.interpolation-cubic-bezier",
        };
        found.insert(id.to_owned());
    }
}

fn constant_eq(value: &Animatable<f64>, expected: f64) -> bool {
    matches!(value, Animatable::Constant { value } if *value == expected)
}
