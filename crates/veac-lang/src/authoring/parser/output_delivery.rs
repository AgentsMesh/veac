mod hls;

use crate::authoring::{
    AnimatedImageRecipe, AudioChannelLayoutDecl, AudioFileRecipe, GifDither, GifPlaybackDecl,
    Mp3EncodingDecl, NumberLiteral, OutputKeyword, SemanticBlock, SemanticEntry, SemanticValue,
    StillImageRecipe,
};

use super::output_fields::{tagged_block, tagged_leaf};
use super::semantic::{finish, number, required, word};
use super::Parser;

impl Parser {
    pub(super) fn audio_file_recipe(
        &mut self,
        body: &mut SemanticBlock,
    ) -> Option<AudioFileRecipe> {
        let source = required(self, body, "source", "audio-file artifact")
            .map(|entry| self.stem_source(entry))?;
        let encode = required(self, body, "encode", "audio-file artifact")?;
        let (format, mut block) = tagged_block(self, encode, "audio-file encode")?;
        if format.value != "mp3" {
            return self.invalid_recipe("audio-file", &format);
        }
        let bitrate = required_number(self, &mut block, "bitrate", "MP3")?;
        let sample_rate = required_number(self, &mut block, "sample-rate", "MP3")?;
        let channel_layout = required_word(self, &mut block, "channel-layout", "MP3")
            .and_then(|value| self.channel_layout(&value))?;
        finish(self, block, "MP3 encode");
        Some(AudioFileRecipe {
            source,
            encoding: Mp3EncodingDecl {
                bitrate,
                sample_rate,
                channel_layout,
            },
        })
    }

    pub(super) fn animated_image_recipe(
        &mut self,
        body: &mut SemanticBlock,
    ) -> Option<AnimatedImageRecipe> {
        let encode = required(self, body, "encode", "animated-image artifact")?;
        let (format, mut block) = tagged_block(self, encode, "animated-image encode")?;
        if format.value != "gif" {
            return self.invalid_recipe("animated-image", &format);
        }
        let playback = required(self, &mut block, "playback", "GIF encode")
            .and_then(|entry| self.gif_playback(&entry))?;
        let dither = required_word(self, &mut block, "dither", "GIF").and_then(|value| {
            match GifDither::parse(&value.value) {
                Some(value) => Some(value),
                None => self.invalid_recipe("GIF dither", &value),
            }
        })?;
        finish(self, block, "GIF encode");
        Some(AnimatedImageRecipe { playback, dither })
    }

    pub(super) fn still_image_recipe(
        &mut self,
        body: &mut SemanticBlock,
    ) -> Option<StillImageRecipe> {
        let frame = required(self, body, "frame", "still-image artifact")?;
        let at = self.containing_frame(&frame)?;
        let encode = required(self, body, "encode", "still-image artifact")?;
        let format = tagged_leaf(self, encode, "still-image encode")?;
        let format = crate::authoring::ImageFormat::parse(&format.value)
            .or_else(|| self.invalid_recipe("still-image", &format))?;
        Some(StillImageRecipe { at, format })
    }

    pub(super) fn channel_layout(
        &mut self,
        value: &crate::authoring::Identifier,
    ) -> Option<AudioChannelLayoutDecl> {
        match value.value.as_str() {
            "mono" => Some(AudioChannelLayoutDecl::Mono),
            "stereo" => Some(AudioChannelLayoutDecl::Stereo),
            _ => self.invalid_recipe("channel-layout", value),
        }
    }

    fn gif_playback(&mut self, entry: &SemanticEntry) -> Option<GifPlaybackDecl> {
        let parsed = match entry.values.as_slice() {
            [SemanticValue::Identifier(value)] if value.value == "once" => {
                Some(GifPlaybackDecl::Once)
            }
            [SemanticValue::Identifier(value)] if value.value == "forever" => {
                Some(GifPlaybackDecl::Forever)
            }
            [SemanticValue::Number(value)] => value
                .raw
                .strip_suffix("times")
                .and_then(|count| count.parse::<u16>().ok())
                .filter(|count| *count >= 2)
                .map(GifPlaybackDecl::Times),
            _ => None,
        };
        if entry.block.is_none() && parsed.is_some() {
            parsed
        } else {
            self.error(
                "AUTHORING_GIF_PLAYBACK",
                "playback must be once, forever, or an integer of at least 2times".into(),
                entry.span,
            );
            None
        }
    }

    pub(super) fn containing_frame(&mut self, entry: &SemanticEntry) -> Option<NumberLiteral> {
        match entry.values.as_slice() {
            [SemanticValue::Identifier(mode), SemanticValue::Number(at)]
                if mode.value == "containing" && entry.block.is_none() =>
            {
                Some(at.clone())
            }
            _ => {
                self.error(
                    "AUTHORING_FRAME_SELECTION",
                    "frame must be `containing <time>`".into(),
                    entry.span,
                );
                None
            }
        }
    }

    pub(super) fn invalid_recipe<T>(
        &mut self,
        context: &str,
        value: &crate::authoring::Identifier,
    ) -> Option<T> {
        self.error(
            "AUTHORING_RECIPE_VARIANT",
            format!("unsupported {context} variant '{}'", value.value),
            value.span,
        );
        None
    }
}

fn required_number(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
    context: &'static str,
) -> Option<NumberLiteral> {
    required(parser, block, name, context).and_then(|entry| number(parser, &entry, name))
}

fn required_word(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
    context: &'static str,
) -> Option<crate::authoring::Identifier> {
    required(parser, block, name, context).and_then(|entry| word(parser, &entry, name))
}
