use super::Parser;
use crate::authoring::{SourceDecl, Span, TypedReference};

impl Parser {
    pub(super) fn source(&mut self) -> Option<SourceDecl> {
        let start = self.required_word("source")?;
        let source_type = self.identifier("source type")?;
        let value = match source_type.value.as_str() {
            "media" => SourceDecl::Media {
                resource: self.source_reference("resource")?,
                span: Span::default(),
            },
            "text" => {
                let text = self.text_source(start)?;
                return Some(SourceDecl::Text {
                    span: text.span,
                    text,
                });
            }
            "caption" => {
                let caption = self.caption_source(start)?;
                return Some(SourceDecl::Caption {
                    span: caption.span,
                    caption,
                });
            }
            "generated" => {
                let generator = self.generator()?;
                return Some(SourceDecl::Generated {
                    span: start.join(generator.span()),
                    generator,
                });
            }
            "sequence" => SourceDecl::Sequence {
                sequence: self.source_reference("sequence")?,
                span: Span::default(),
            },
            "multicam" => return self.multicam_source(start),
            _ => {
                self.error(
                    "AUTHORING_SOURCE_TYPE",
                    "source type must be media, text, caption, generated, sequence, or multicam"
                        .to_owned(),
                    source_type.span,
                );
                self.recover_declaration();
                return None;
            }
        };
        let end = self.semicolon()?;
        Some(with_span(value, start.join(end)))
    }

    fn source_reference(&mut self, expected: &'static str) -> Option<TypedReference> {
        let reference = self.typed_reference()?;
        if reference.kind.value != expected {
            self.error(
                "AUTHORING_REFERENCE_KIND",
                format!("source requires `{expected} <id>`"),
                reference.kind.span,
            );
            return None;
        }
        Some(reference)
    }
}

fn with_span(value: SourceDecl, span: Span) -> SourceDecl {
    match value {
        SourceDecl::Media { resource, .. } => SourceDecl::Media { resource, span },
        SourceDecl::Text { text, .. } => SourceDecl::Text { text, span },
        SourceDecl::Caption { caption, .. } => SourceDecl::Caption { caption, span },
        SourceDecl::Generated { generator, .. } => SourceDecl::Generated { generator, span },
        SourceDecl::Sequence { sequence, .. } => SourceDecl::Sequence { sequence, span },
        SourceDecl::Multicam {
            group, switches, ..
        } => SourceDecl::Multicam {
            group,
            switches,
            span,
        },
    }
}
