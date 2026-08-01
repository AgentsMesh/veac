mod options;

use crate::authoring::{
    AudioCodec, AudioOutput, HardwareBackend, HardwareSelection, OutputFormat, OutputKeyword,
    SemanticBlock, SemanticEntry, SemanticValue, VideoCodec, VideoEncoding, VideoOutput,
};

use super::output_fields::*;
use super::semantic::{finish, number, required, take, word};
use super::Parser;

impl Parser {
    pub(super) fn video_recipe(&mut self, body: &mut SemanticBlock) -> Option<VideoEncoding> {
        let entry = required(self, body, "mux", "video artifact")?;
        let (container, mut mux) = tagged_block(self, entry, "video mux")?;
        let container = OutputFormat::parse(&container.value)
            .or_else(|| self.invalid_recipe("video mux", &container))?;
        let mut output = VideoEncoding {
            container,
            ..VideoEncoding::default()
        };
        if let Some(layout) = take_word(self, &mut mux, "layout") {
            output.optimize_for_streaming = match layout.value.as_str() {
                "standard" => false,
                "fast-start" => true,
                _ => return self.invalid_recipe("mux layout", &layout),
            };
        }
        let video = required(self, &mut mux, "video", "video mux")?;
        let (codec, settings) = tagged_block(self, video, "video stream")?;
        output.video.codec = VideoCodec::parse(&codec.value)
            .or_else(|| self.invalid_recipe("video codec", &codec))?;
        self.video_settings(settings, &mut output.video);
        let audio = required(self, &mut mux, "audio", "video mux")?;
        output.audio = self.optional_audio_recipe(audio)?;
        field_enum(self, &mut mux, "passes", &mut output.pass_mode);
        if let Some(value) = take_word(self, &mut mux, "accelerator") {
            output.hardware = self.hardware(&value)?;
        }
        finish(self, mux, "video mux");
        Some(output)
    }

    pub(super) fn video_settings(&mut self, mut block: SemanticBlock, output: &mut VideoOutput) {
        field_enum(self, &mut block, "pixel-format", &mut output.pixel_format);
        field_enum(self, &mut block, "alpha", &mut output.alpha);
        if let Some(entry) = take(self, &mut block, "color-space") {
            output.color_space = match (entry.values.as_slice(), entry.block) {
                ([SemanticValue::Identifier(value)], None) if value.value == "source" => None,
                ([], Some(block)) => Some(self.color_space(block)),
                _ => {
                    self.error(
                        "AUTHORING_OUTPUT_COLOR_SPACE",
                        "color-space must be source or a settings block".into(),
                        entry.span,
                    );
                    output.color_space
                }
            };
        }
        if let Some(entry) = take(self, &mut block, "rate-control") {
            output.rate_control = self.rate_control(entry);
        }
        output.gop_size = self.optional_u32(&mut block, "gop", output.gop_size);
        output.b_frames = self.optional_u8(&mut block, "b-frames", output.b_frames);
        if let Some(entry) = take(self, &mut block, "profile") {
            output.profile = self.optional_profile(entry, output.profile);
        }
        if let Some(entry) = take(self, &mut block, "level") {
            output.level = self.optional_level(entry, output.level.clone());
        }
        finish(self, block, "video encode");
    }

    pub(super) fn audio_settings(
        &mut self,
        codec: AudioCodec,
        mut block: SemanticBlock,
    ) -> Option<AudioOutput> {
        let sample = required(self, &mut block, "sample-rate", "audio encode")
            .and_then(|entry| number(self, &entry, "audio sample-rate"))?;
        let sample_rate = sample_rate(self, &sample, "audio sample-rate")?;
        let layout = required(self, &mut block, "channel-layout", "audio encode")
            .and_then(|entry| word(self, &entry, "audio channel-layout"))?;
        let channels = self.output_channel_count(&layout)?;
        finish(self, block, "audio encode");
        Some(AudioOutput {
            codec,
            sample_rate,
            channels,
        })
    }

    fn optional_audio_recipe(&mut self, entry: SemanticEntry) -> Option<Option<AudioOutput>> {
        match (entry.values.as_slice(), entry.block) {
            ([SemanticValue::Identifier(value)], None) if value.value == "none" => Some(None),
            ([SemanticValue::Identifier(codec)], Some(block)) => {
                let codec = AudioCodec::parse(&codec.value)
                    .or_else(|| self.invalid_recipe("audio codec", codec))?;
                self.audio_settings(codec, block).map(Some)
            }
            _ => {
                self.error(
                    "AUTHORING_OUTPUT_AUDIO",
                    "audio must be `none` or `<codec> { ... }`".into(),
                    entry.span,
                );
                None
            }
        }
    }

    fn hardware(&mut self, value: &crate::authoring::Identifier) -> Option<HardwareSelection> {
        match value.value.as_str() {
            "auto" => Some(HardwareSelection::Auto),
            "software" => Some(HardwareSelection::Software),
            _ => HardwareBackend::parse(&value.value)
                .map(|backend| HardwareSelection::Explicit { backend })
                .or_else(|| self.invalid_recipe("accelerator", value)),
        }
    }

    fn output_channel_count(&mut self, value: &crate::authoring::Identifier) -> Option<u8> {
        match value.value.as_str() {
            "mono" => Some(1),
            "stereo" => Some(2),
            "discrete-3" => Some(3),
            "discrete-4" => Some(4),
            "discrete-5" => Some(5),
            "surround-5-1" => Some(6),
            "discrete-7" => Some(7),
            "surround-7-1" => Some(8),
            _ => self.invalid_recipe("audio channel-layout", value),
        }
    }
}
