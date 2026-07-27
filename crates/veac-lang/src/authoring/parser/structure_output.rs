use crate::authoring::ast::*;
use crate::authoring::{Span, Spanned};

use super::output_fields::{is_leaf_name, string};
use super::semantic::{finish, nested, required, word};
use super::Parser;

impl Parser {
    pub(super) fn output(&mut self) -> Option<OutputDecl> {
        let start = self.advance().span.start;
        let kind = self.identifier("output kind")?;
        let id = self.identifier("output id")?;
        let mut body = self.semantic_block()?;
        let sequence = required(self, &mut body, "sequence", "output")
            .and_then(|entry| word(self, &entry, "sequence"))?;
        let file_name = required(self, &mut body, "file-name", "output")
            .and_then(|entry| string(self, &entry, "file-name"))?;
        self.validate_file_name(&file_name);
        let encoding = required(self, &mut body, "encoding", "output")
            .and_then(|entry| nested(self, &entry, "encoding"))
            .and_then(|block| self.output_encoding(&kind, block))?;
        let end = body.span.end;
        finish(self, body, "output");
        Some(OutputDecl {
            id,
            sequence,
            file_name,
            encoding,
            span: Span { start, end },
        })
    }

    fn output_encoding(
        &mut self,
        kind: &Identifier,
        block: SemanticBlock,
    ) -> Option<OutputEncoding> {
        let value = match kind.value.as_str() {
            "video" => OutputEncoding::Video(self.video_encoding(block)),
            "image-sequence" => OutputEncoding::ImageSequence(self.image_encoding(block)),
            "caption-sidecar" => OutputEncoding::CaptionSidecar(self.caption_encoding(block)),
            "audio-stem" => OutputEncoding::AudioStem(self.audio_stem_encoding(block)),
            "scope" => OutputEncoding::Scope(self.scope_encoding(block)),
            other => {
                self.error(
                    "AUTHORING_OUTPUT_KIND",
                    format!("unsupported output kind '{other}'"),
                    kind.span,
                );
                return None;
            }
        };
        Some(value)
    }

    fn validate_file_name(&mut self, value: &Spanned<String>) {
        if !is_leaf_name(&value.value) {
            self.error(
                "AUTHORING_OUTPUT_FILE_NAME",
                "file-name must be a leaf basename".into(),
                value.span,
            );
        }
    }
}
