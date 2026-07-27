use crate::authoring::{AudioFadeCurveDecl, AudioModifierDecl, PitchPolicyDecl};

use super::audio_processor::processor;
use super::parameter::scalar;
use super::writer::Writer;

pub(super) fn audio(writer: &mut Writer, value: &AudioModifierDecl) {
    writer.block(format!("audio {}", value.id.value), |writer| {
        if let Some(value) = &value.gain {
            scalar(writer, "gain", value);
        }
        if let Some(value) = &value.pan {
            scalar(writer, "pan", value);
        }
        if let Some(value) = &value.muted {
            writer.line(format!("muted {};", value.value));
        }
        if let Some(value) = &value.normalize {
            writer.line(format!("normalize {};", value.value));
        }
        if let Some(value) = &value.pitch {
            writer.line(format!(
                "pitch {};",
                match value.value {
                    PitchPolicyDecl::Preserve => "preserve",
                    PitchPolicyDecl::FollowSpeed => "follow-speed",
                }
            ));
        }
        for value in &value.processors {
            processor(writer, value);
        }
        if let Some(value) = &value.crossfade {
            writer.block("crossfade", |writer| {
                writer.line(format!("fade-in {};", value.fade_in.raw));
                writer.line(format!("fade-out {};", value.fade_out.raw));
                writer.line(format!(
                    "curve {};",
                    match value.curve.value {
                        AudioFadeCurveDecl::Linear => "linear",
                        AudioFadeCurveDecl::EqualPower => "equal-power",
                        AudioFadeCurveDecl::Exponential => "exponential",
                    }
                ));
            });
        }
    });
}
