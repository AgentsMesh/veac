use veac_artifact::{BoundAudioFacts, BoundVideoFacts};
use veac_plan::canonical::{
    samples_for_duration, units_for_duration, Deliverable, Rational, RationalTime,
    SequenceSettings, VideoCadence, MAX_REVERSE_BUFFERED_BYTES,
    REVERSE_DECODED_BYTES_PER_CHANNEL_SAMPLE, REVERSE_DECODED_BYTES_PER_PIXEL,
    REVERSE_FRAME_OVERHEAD_BYTES,
};
use veac_plan::ResolvedClip;

use super::error::{diagnostic, CodegenErrorKind};
use super::{CodegenErrors, EmitContext};

#[derive(Default)]
pub(super) struct ReverseLedger {
    bytes: u128,
    instances: u64,
}

#[derive(Clone, Copy)]
pub(super) struct VideoFacts {
    width: u32,
    height: u32,
    frame_rate: Option<Rational>,
    cadence: VideoCadence,
}

#[derive(Clone, Copy)]
pub(super) struct AudioFacts {
    sample_rate: u32,
    channels: u8,
}

impl VideoFacts {
    pub(super) fn bound(value: BoundVideoFacts) -> Self {
        Self {
            width: value.width(),
            height: value.height(),
            frame_rate: value.frame_rate(),
            cadence: value.cadence(),
        }
    }

    pub(super) fn sequence(value: &SequenceSettings) -> Self {
        Self {
            width: value.width,
            height: value.height,
            frame_rate: Some(value.frame_rate),
            cadence: VideoCadence::Constant,
        }
    }
}

impl AudioFacts {
    pub(super) fn bound(value: BoundAudioFacts) -> Self {
        Self {
            sample_rate: value.sample_rate(),
            channels: value.channels(),
        }
    }

    pub(super) fn output(sample_rate: u32, channels: u8) -> Self {
        Self {
            sample_rate,
            channels,
        }
    }
}

impl EmitContext<'_> {
    pub(super) fn reverse_video(
        &mut self,
        input: &str,
        prefix: &str,
        clip: &ResolvedClip,
        span: RationalTime,
        facts: Option<VideoFacts>,
    ) -> Result<String, CodegenErrors> {
        self.reverse_ledger.video(clip, span, facts)?;
        Ok(self.graph.filter(&[input], "reverse", prefix))
    }

    pub(super) fn reverse_audio(
        &mut self,
        input: &str,
        prefix: &str,
        clip: &ResolvedClip,
        span: RationalTime,
        output_rate: u32,
        facts: Option<AudioFacts>,
    ) -> Result<String, CodegenErrors> {
        self.reverse_ledger.audio(clip, span, output_rate, facts)?;
        Ok(self.graph.filter(&[input], "areverse", prefix))
    }
}

impl ReverseLedger {
    fn video(
        &mut self,
        clip: &ResolvedClip,
        span: RationalTime,
        facts: Option<VideoFacts>,
    ) -> Result<(), CodegenErrors> {
        let Some(facts) = facts else {
            return Err(facts_error(clip, "video reverse facts are missing"));
        };
        if facts.cadence != VideoCadence::Constant {
            return Err(cadence_error(clip));
        }
        let Some(rate) = facts.frame_rate.filter(|rate| rate.is_positive()) else {
            return Err(facts_error(clip, "video reverse frame rate is missing"));
        };
        if facts.width == 0 || facts.height == 0 {
            return Err(facts_error(clip, "video reverse dimensions are missing"));
        }
        let frames = units_for_duration(span, rate)
            .map(|value| value.saturating_add(1))
            .unwrap_or(u128::MAX);
        let bytes = frames
            .saturating_mul(u128::from(facts.width))
            .saturating_mul(u128::from(facts.height))
            .saturating_mul(REVERSE_DECODED_BYTES_PER_PIXEL)
            .saturating_add(frames.saturating_mul(REVERSE_FRAME_OVERHEAD_BYTES));
        self.bytes = self.bytes.saturating_add(bytes);
        self.instances = self.instances.saturating_add(1);
        Ok(())
    }

    fn audio(
        &mut self,
        clip: &ResolvedClip,
        span: RationalTime,
        output_rate: u32,
        facts: Option<AudioFacts>,
    ) -> Result<(), CodegenErrors> {
        let Some(facts) = facts.filter(|value| value.sample_rate > 0 && value.channels > 0) else {
            return Err(facts_error(clip, "audio reverse facts are missing"));
        };
        let bytes = samples_for_duration(span, output_rate)
            .unwrap_or(u128::MAX)
            .saturating_mul(u128::from(facts.channels))
            .saturating_mul(REVERSE_DECODED_BYTES_PER_CHANNEL_SAMPLE);
        self.bytes = self.bytes.saturating_add(bytes);
        self.instances = self.instances.saturating_add(1);
        Ok(())
    }

    pub(super) fn validate(
        &self,
        deliverable: &Deliverable,
        emitted: u64,
    ) -> Result<(), CodegenErrors> {
        if emitted != self.instances {
            return Err(CodegenErrors::one(diagnostic(
                CodegenErrorKind::InvalidPlan,
                "BACKEND_REVERSE_BUDGET_BYPASS",
                Some(deliverable.id.to_string()),
                "every emitted reverse filter must be charged through the reverse ledger",
            )));
        }
        if self.bytes <= MAX_REVERSE_BUFFERED_BYTES {
            return Ok(());
        }
        Err(CodegenErrors::one(diagnostic(
            CodegenErrorKind::InvalidPlan,
            "PLAN_BUDGET_REVERSE_BYTES",
            Some(deliverable.id.to_string()),
            "reverse filters exceed the conservative decoded-buffer byte budget",
        )))
    }
}

fn facts_error(clip: &ResolvedClip, message: &'static str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidPlan,
        "PLAN_REVERSE_SOURCE_FACTS_MISSING",
        Some(clip.id.to_string()),
        message,
    ))
}

fn cadence_error(clip: &ResolvedClip) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidPlan,
        "PLAN_REVERSE_CADENCE_UNSUPPORTED",
        Some(clip.id.to_string()),
        "video reverse requires complete packet timestamps with constant cadence; bind a verified CFR proxy or transcode the source to CFR and reprobe it",
    ))
}

#[cfg(test)]
#[path = "reverse_ledger/tests.rs"]
mod tests;
