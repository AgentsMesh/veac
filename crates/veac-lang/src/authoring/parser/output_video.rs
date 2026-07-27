use crate::authoring::{
    AudioCodec, AudioOutput, HardwareBackend, HardwareSelection, OutputKeyword, SemanticBlock,
    SemanticEntry, SemanticValue, VideoEncoding, VideoOutput,
};

use super::output_fields::*;
use super::semantic::{finish, take};
use super::Parser;

impl Parser {
    pub(super) fn video_encoding(&mut self, mut block: SemanticBlock) -> VideoEncoding {
        let mut output = VideoEncoding::default();
        field_enum(self, &mut block, "container", &mut output.container);
        if let Some(video) = take_block(self, &mut block, "video") {
            self.video_settings(video, &mut output.video);
        }
        if let Some(entry) = take(self, &mut block, "audio") {
            output.audio = self.optional_audio(entry, output.audio);
        }
        field_enum(self, &mut block, "captions", &mut output.captions);
        field_bool(
            self,
            &mut block,
            "optimize-for-streaming",
            &mut output.optimize_for_streaming,
        );
        field_enum(self, &mut block, "pass-mode", &mut output.pass_mode);
        if let Some(value) = take_word(self, &mut block, "hardware") {
            output.hardware = match value.value.as_str() {
                "auto" => HardwareSelection::Auto,
                "software" => HardwareSelection::Software,
                _ => match HardwareBackend::parse(&value.value) {
                    Some(backend) => HardwareSelection::Explicit { backend },
                    None => {
                        self.error(
                            "AUTHORING_OUTPUT_ENUM",
                            format!("invalid hardware value '{}'", value.value),
                            value.span,
                        );
                        output.hardware
                    }
                },
            };
        }
        finish(self, block, "video encoding");
        output
    }

    fn video_settings(&mut self, mut block: SemanticBlock, output: &mut VideoOutput) {
        field_enum(self, &mut block, "codec", &mut output.codec);
        field_enum(self, &mut block, "pixel-format", &mut output.pixel_format);
        field_enum(self, &mut block, "alpha", &mut output.alpha);
        if let Some(value) = take_block(self, &mut block, "color-space") {
            output.color_space = Some(self.color_space(value));
        }
        if let Some(entry) = take(self, &mut block, "rate-control") {
            output.rate_control = self.rate_control(entry);
        }
        if let Some(value) = take_u64(self, &mut block, "gop-size") {
            output.gop_size = u32::try_from(value).ok();
        }
        if let Some(value) = take_u64(self, &mut block, "b-frames") {
            output.b_frames = u8::try_from(value).ok();
        }
        field_optional_enum(self, &mut block, "profile", &mut output.profile);
        field_string(self, &mut block, "level", &mut output.level);
        finish(self, block, "video settings");
    }

    fn optional_audio(
        &mut self,
        entry: SemanticEntry,
        fallback: Option<AudioOutput>,
    ) -> Option<AudioOutput> {
        match (entry.values.as_slice(), entry.block) {
            ([SemanticValue::Identifier(value)], None) if value.value == "none" => None,
            ([], Some(block)) => Some(self.audio_settings(block)),
            _ => {
                self.error(
                    "AUTHORING_OUTPUT_AUDIO",
                    "audio must be `none` or a settings block".into(),
                    entry.span,
                );
                fallback
            }
        }
    }

    pub(super) fn audio_settings(&mut self, mut block: SemanticBlock) -> AudioOutput {
        let mut output = AudioOutput {
            codec: AudioCodec::Aac,
            sample_rate: 48_000,
            channels: 2,
        };
        field_enum(self, &mut block, "codec", &mut output.codec);
        field_u32(self, &mut block, "sample-rate", &mut output.sample_rate);
        field_u8(self, &mut block, "channels", &mut output.channels);
        finish(self, block, "audio settings");
        output
    }
}
