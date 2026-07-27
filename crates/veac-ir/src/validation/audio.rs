use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn audio(
        &mut self,
        audio: &AudioProperties,
        duration: RationalTime,
        timebase: u32,
        sample_rate: u32,
        path: &str,
        item_id: &str,
    ) {
        self.animatable(
            &audio.gain,
            duration,
            timebase,
            &format!("{path}/audio/gain"),
            item_id,
            |value| value.is_finite() && *value >= 0.0,
        );
        self.animatable(
            &audio.pan,
            duration,
            timebase,
            &format!("{path}/audio/pan"),
            item_id,
            |value| value.is_finite() && (-1.0..=1.0).contains(value),
        );
        self.processors(audio, sample_rate, path, item_id);
        self.crossfade(audio.crossfade, duration, timebase, path, item_id);
    }

    fn processors(&mut self, audio: &AudioProperties, rate: u32, path: &str, id: &str) {
        if audio.processors.len() > 64 {
            self.value_error("AUDIO_PROCESSOR_COUNT", path, id);
        }
        let loudness = audio
            .processors
            .iter()
            .filter(|value| matches!(value, AudioProcessor::Loudness(_)))
            .count();
        if loudness > 1 || (audio.normalize && loudness > 0) {
            self.value_error("AUDIO_PROCESSOR_CONFLICT", path, id);
        }
        for (index, processor) in audio.processors.iter().enumerate() {
            let processor_path = format!("{path}/audio/processors/{index}");
            if !audio_processor_valid(processor, rate) {
                self.value_error("AUDIO_PROCESSOR", &processor_path, id);
            }
        }
    }

    fn crossfade(
        &mut self,
        value: Option<AudioCrossfade>,
        duration: RationalTime,
        timebase: u32,
        path: &str,
        id: &str,
    ) {
        let Some(value) = value else { return };
        self.time(value.fade_in, timebase, false, "AUDIO_CROSSFADE", path, id);
        self.time(value.fade_out, timebase, false, "AUDIO_CROSSFADE", path, id);
        let total = value.fade_in.value.checked_add(value.fade_out.value);
        if value.fade_in.timescale != value.fade_out.timescale
            || total.is_none_or(|total| total > duration.value)
        {
            self.value_error("AUDIO_CROSSFADE", path, id);
        }
    }

    pub(super) fn sidechain(
        &mut self,
        value: &SidechainRelationParameters,
        duration: RationalTime,
        timebase: u32,
        path: &str,
        id: &str,
    ) {
        if !db(value.threshold_db, -60.0, 0.0)
            || !finite_range(value.ratio, 1.0, 20.0)
            || !finite_range(value.attack_ms, 0.01, 2000.0)
            || !finite_range(value.release_ms, 0.01, 9000.0)
        {
            self.value_error("SIDECHAIN_PARAMETERS", path, id);
        }
        if let Some(range) = value.active_range {
            self.time_range(range, timebase, "SIDECHAIN_RANGE", path, id);
            if range.end().is_err() || range.end().is_ok_and(|end| end > duration) {
                self.value_error("SIDECHAIN_RANGE", path, id);
            }
        }
    }
}

fn db(value: f64, minimum: f64, maximum: f64) -> bool {
    finite_range(value, minimum, maximum)
}

fn finite_range(value: f64, minimum: f64, maximum: f64) -> bool {
    value.is_finite() && (minimum..=maximum).contains(&value)
}
