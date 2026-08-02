use crate::authoring::{
    AudioCodec, AudioMixSourceDecl, AudioStemEncoding, AudioStemFormat, CaptionSidecarEncoding,
    CaptionSidecarFormat, ImageFormat, ImageSequenceEncoding, OutputKeyword, ScopeEncoding,
    SemanticBlock, SemanticEntry, SemanticValue, VideoScope,
};

use super::output_fields::{pixels, tagged_block, tagged_leaf};
use super::semantic::{required, word};
use super::Parser;

impl Parser {
    pub(super) fn image_sequence_recipe(
        &mut self,
        body: &mut SemanticBlock,
    ) -> Option<ImageSequenceEncoding> {
        let numbering = required(self, body, "numbering", "image-sequence artifact")?;
        let start_number = self.numbering(&numbering)?;
        let encode = required(self, body, "encode", "image-sequence artifact")?;
        let format = tagged_leaf(self, encode, "image-sequence encode")?;
        let format = ImageFormat::parse(&format.value)
            .or_else(|| self.invalid_recipe("image-sequence", &format))?;
        Some(ImageSequenceEncoding {
            format,
            start_number,
        })
    }

    pub(super) fn caption_sidecar_recipe(
        &mut self,
        body: &mut SemanticBlock,
    ) -> Option<CaptionSidecarEncoding> {
        let source = required(self, body, "source", "caption-sidecar artifact")?;
        let track_ids = self.caption_tracks(source)?;
        let encode = required(self, body, "encode", "caption-sidecar artifact")?;
        let format = tagged_leaf(self, encode, "caption-sidecar encode")?;
        let format = CaptionSidecarFormat::parse(&format.value)
            .or_else(|| self.invalid_recipe("caption-sidecar", &format))?;
        Some(CaptionSidecarEncoding { format, track_ids })
    }

    pub(super) fn audio_stem_recipe(
        &mut self,
        body: &mut SemanticBlock,
    ) -> Option<AudioStemEncoding> {
        let source = required(self, body, "source", "audio-stem artifact")
            .map(|entry| self.stem_source(entry))?;
        let encode = required(self, body, "encode", "audio-stem artifact")?;
        let (format, mut settings) = tagged_block(self, encode, "audio-stem encode")?;
        let (format, codec) = match format.value.as_str() {
            "wav" => {
                let entry = required(self, &mut settings, "sample-format", "WAV encode")?;
                let sample = word(self, &entry, "WAV sample-format")?;
                let codec = match AudioCodec::parse(&sample.value) {
                    Some(
                        value
                        @ (AudioCodec::PcmS16Le | AudioCodec::PcmS24Le | AudioCodec::PcmS32Le),
                    ) => value,
                    _ => return self.invalid_recipe("WAV sample-format", &sample),
                };
                (AudioStemFormat::Wav, codec)
            }
            "flac" => (AudioStemFormat::Flac, AudioCodec::Flac),
            _ => return self.invalid_recipe("audio-stem", &format),
        };
        let audio = self.audio_settings(codec, settings)?;
        Some(AudioStemEncoding {
            format,
            audio,
            source,
        })
    }

    pub(super) fn scope_recipe(&mut self, body: &mut SemanticBlock) -> Option<ScopeEncoding> {
        let analyze = required(self, body, "analyze", "scope artifact")?;
        let scope = tagged_leaf(self, analyze, "scope analysis")?;
        let scope = VideoScope::parse(&scope.value)
            .or_else(|| self.invalid_recipe("scope analysis", &scope))?;
        let frame = required(self, body, "frame", "scope artifact")?;
        let at = self.containing_frame(&frame)?;
        let canvas = required(self, body, "canvas", "scope artifact")?;
        let (width, height) = self.scope_canvas(&canvas)?;
        let encode = required(self, body, "encode", "scope artifact")?;
        let format = tagged_leaf(self, encode, "scope encode")?;
        let format =
            ImageFormat::parse(&format.value).or_else(|| self.invalid_recipe("scope", &format))?;
        Some(ScopeEncoding {
            scope,
            at,
            width,
            height,
            format,
        })
    }

    pub(super) fn stem_source(&mut self, entry: SemanticEntry) -> AudioMixSourceDecl {
        let parsed = match entry.values.as_slice() {
            [SemanticValue::Identifier(kind)] if kind.value == "master" => {
                Some(AudioMixSourceDecl::Master)
            }
            [SemanticValue::Identifier(kind), SemanticValue::Identifier(id)]
                if kind.value == "track" =>
            {
                Some(AudioMixSourceDecl::Track(id.clone()))
            }
            [SemanticValue::Identifier(kind), SemanticValue::Identifier(id)]
                if kind.value == "bus" =>
            {
                Some(AudioMixSourceDecl::Bus(id.clone()))
            }
            _ => None,
        };
        if entry.block.is_some() || parsed.is_none() {
            self.error(
                "AUTHORING_OUTPUT_SOURCE",
                "source must be master, track <id>, or bus <id>".into(),
                entry.span,
            );
        }
        parsed.unwrap_or(AudioMixSourceDecl::Master)
    }

    fn numbering(&mut self, entry: &SemanticEntry) -> Option<u32> {
        match entry.values.as_slice() {
            [SemanticValue::Identifier(from), SemanticValue::Number(value)]
                if from.value == "from" && entry.block.is_none() =>
            {
                value.raw.parse().ok()
            }
            _ => {
                self.error(
                    "AUTHORING_IMAGE_NUMBERING",
                    "numbering must be `from <unsigned-integer>`".into(),
                    entry.span,
                );
                None
            }
        }
    }

    fn caption_tracks(
        &mut self,
        entry: SemanticEntry,
    ) -> Option<Vec<crate::authoring::Identifier>> {
        let ([SemanticValue::Identifier(kind)], Some(block)) =
            (entry.values.as_slice(), entry.block)
        else {
            self.error(
                "AUTHORING_CAPTION_SOURCE",
                "caption source must be `caption-tracks { ... }`".into(),
                entry.span,
            );
            return None;
        };
        if kind.value != "caption-tracks" {
            return self.invalid_recipe("caption source", kind);
        }
        let mut tracks = Vec::new();
        for entry in block.entries {
            match entry.values.as_slice() {
                [SemanticValue::Identifier(id)]
                    if entry.name.value == "track" && entry.block.is_none() =>
                {
                    tracks.push(id.clone());
                }
                _ => self.error(
                    "AUTHORING_CAPTION_SOURCE",
                    "caption-tracks only accepts `track <id>;`".into(),
                    entry.span,
                ),
            }
        }
        Some(tracks)
    }

    fn scope_canvas(&mut self, entry: &SemanticEntry) -> Option<(u32, u32)> {
        match entry.values.as_slice() {
            [SemanticValue::Number(width), SemanticValue::Identifier(by), SemanticValue::Number(height)]
                if by.value == "by" && entry.block.is_none() =>
            {
                Some((
                    pixels(self, width, "scope width")?,
                    pixels(self, height, "scope height")?,
                ))
            }
            _ => {
                self.error(
                    "AUTHORING_SCOPE_CANVAS",
                    "scope canvas must be `<width>px by <height>px`".into(),
                    entry.span,
                );
                None
            }
        }
    }
}
