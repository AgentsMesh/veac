use crate::authoring::{
    AudioStemEncoding, AudioStemSourceDecl, CaptionSidecarEncoding, ImageSequenceEncoding,
    ScopeEncoding, SemanticBlock, SemanticValue,
};

use super::output_fields::*;
use super::semantic::{finish, take};
use super::Parser;

impl Parser {
    pub(super) fn image_encoding(&mut self, mut block: SemanticBlock) -> ImageSequenceEncoding {
        let mut output = ImageSequenceEncoding::default();
        field_enum(self, &mut block, "format", &mut output.format);
        field_u32(self, &mut block, "start-number", &mut output.start_number);
        finish(self, block, "image-sequence encoding");
        output
    }

    pub(super) fn caption_encoding(&mut self, mut block: SemanticBlock) -> CaptionSidecarEncoding {
        let mut output = CaptionSidecarEncoding::default();
        field_enum(self, &mut block, "format", &mut output.format);
        if let Some(tracks) = take_block(self, &mut block, "tracks") {
            for entry in tracks.entries {
                if entry.name.value != "track" {
                    self.error(
                        "AUTHORING_UNKNOWN_FIELD",
                        "tracks only accepts track entries".into(),
                        entry.name.span,
                    );
                } else if let Some(id) = entry.values.first().and_then(identifier) {
                    output.track_ids.push(id.clone());
                } else {
                    self.error(
                        "AUTHORING_FIELD_SHAPE",
                        "track requires one identifier".into(),
                        entry.span,
                    );
                }
            }
        }
        finish(self, block, "caption-sidecar encoding");
        output
    }

    pub(super) fn audio_stem_encoding(&mut self, mut block: SemanticBlock) -> AudioStemEncoding {
        let mut output = AudioStemEncoding::default();
        field_enum(self, &mut block, "format", &mut output.format);
        if let Some(audio) = take_block(self, &mut block, "audio") {
            output.audio = self.audio_settings(audio);
        }
        if let Some(entry) = take(self, &mut block, "source") {
            output.source = self.stem_source(entry);
        }
        finish(self, block, "audio-stem encoding");
        output
    }

    fn stem_source(&mut self, entry: crate::authoring::SemanticEntry) -> AudioStemSourceDecl {
        let values = &entry.values;
        let parsed = match values.as_slice() {
            [SemanticValue::Identifier(kind)] if kind.value == "master" => {
                Some(AudioStemSourceDecl::Master)
            }
            [SemanticValue::Identifier(kind), SemanticValue::Identifier(id)]
                if kind.value == "track" =>
            {
                Some(AudioStemSourceDecl::Track(id.clone()))
            }
            [SemanticValue::Identifier(kind), SemanticValue::Identifier(id)]
                if kind.value == "bus" =>
            {
                Some(AudioStemSourceDecl::Bus(id.clone()))
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
        parsed.unwrap_or(AudioStemSourceDecl::Master)
    }

    pub(super) fn scope_encoding(&mut self, mut block: SemanticBlock) -> ScopeEncoding {
        let mut output = ScopeEncoding::default();
        field_enum(self, &mut block, "scope", &mut output.scope);
        if let Some(value) = take_number(self, &mut block, "at") {
            output.at = value;
        }
        field_u32(self, &mut block, "width", &mut output.width);
        field_u32(self, &mut block, "height", &mut output.height);
        field_enum(self, &mut block, "format", &mut output.format);
        finish(self, block, "scope encoding");
        output
    }
}

fn identifier(value: &SemanticValue) -> Option<&crate::authoring::Identifier> {
    match value {
        SemanticValue::Identifier(value) => Some(value),
        _ => None,
    }
}
