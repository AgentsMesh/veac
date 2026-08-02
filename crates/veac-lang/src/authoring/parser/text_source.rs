use crate::authoring::{
    CaptionSourceDecl, SemanticBlock, Span, Spanned, TextLayoutDecl, TextSourceDecl, TextStyleDecl,
};

use super::{semantic, text_animation, text_layout, text_style, text_value, Parser};

impl Parser {
    pub(super) fn text_source(&mut self, start: crate::authoring::Span) -> Option<TextSourceDecl> {
        let body = self.semantic_block()?;
        parse_text(self, start, body, "text source")
    }

    pub(super) fn caption_source(&mut self, start: Span) -> Option<CaptionSourceDecl> {
        let mut body = self.semantic_block()?;
        let speaker = match semantic::take(self, &mut body, "speaker") {
            Some(entry) => Some(text_value::string(self, &entry)?),
            None => None,
        };
        validate_speaker(self, speaker.as_ref());
        let text = parse_text(self, start, body, "caption source")?;
        let span = text.span;
        Some(CaptionSourceDecl {
            text,
            speaker,
            span,
        })
    }
}

fn parse_text(
    parser: &mut Parser,
    start: Span,
    mut body: SemanticBlock,
    context: &'static str,
) -> Option<TextSourceDecl> {
    let content_entry = semantic::required(parser, &mut body, "content", context)?;
    let content = text_value::string(parser, &content_entry)?;
    let style = match semantic::take(parser, &mut body, "style") {
        Some(entry) => text_style::parse(parser, &entry)?,
        None => TextStyleDecl::default(),
    };
    let layout = match semantic::take(parser, &mut body, "layout") {
        Some(entry) => text_layout::parse(parser, &entry)?,
        None => TextLayoutDecl::default(),
    };
    let animation = match semantic::take(parser, &mut body, "animation") {
        Some(entry) => Some(text_animation::parse(parser, &entry)?),
        None => None,
    };
    let span = start.join(body.span);
    semantic::finish(parser, body, context);
    Some(TextSourceDecl {
        content,
        style,
        layout,
        animation,
        span,
    })
}

fn validate_speaker(parser: &mut Parser, value: Option<&Spanned<String>>) {
    let Some(value) = value else { return };
    let invalid = value.value.trim().is_empty()
        || value.value.chars().count() > 256
        || value
            .value
            .chars()
            .any(|character| matches!(character, '\0' | '\r' | '\n'));
    if invalid {
        parser.error(
            "AUTHORING_CAPTION_SPEAKER",
            "caption speaker must be 1..=256 characters on one line".to_owned(),
            value.span,
        );
    }
}
