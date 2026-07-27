use crate::authoring::{Span, Spanned, TemplateFillDecl, TemplateMediaKindDecl, TemplateSlotDecl};

use super::{semantic, text_value, Parser};

impl Parser {
    pub(super) fn template_slot(&mut self) -> Option<TemplateSlotDecl> {
        let start = self.required_word("template-slot")?;
        let kind = self.identifier("template slot kind")?;
        match kind.value.as_str() {
            "text" => {
                let end = self.semicolon()?;
                Some(TemplateSlotDecl::Text {
                    span: start.join(end),
                })
            }
            "media" => self.media_template_slot(start),
            _ => {
                self.error(
                    "AUTHORING_TEMPLATE_SLOT_KIND",
                    "template slot kind must be media or text".to_owned(),
                    kind.span,
                );
                None
            }
        }
    }

    fn media_template_slot(&mut self, start: Span) -> Option<TemplateSlotDecl> {
        let mut body = self.semantic_block()?;
        let accepts = semantic::required(self, &mut body, "accepts", "media template slot")?;
        let accepts = semantic::word(self, &accepts, "slot accepts")?;
        let accepts = parse_accepts(self, accepts)?;
        let fill = semantic::required(self, &mut body, "fill", "media template slot")?;
        let fill = semantic::word(self, &fill, "slot fill")?;
        let fill = parse_fill(self, fill)?;
        let label = semantic::required(self, &mut body, "label", "media template slot")?;
        let label = text_value::string(self, &label)?;
        let minimum_source_duration = semantic::take(self, &mut body, "minimum-source-duration")
            .and_then(|entry| semantic::number(self, &entry, "minimum source duration"));
        let span = start.join(body.span);
        semantic::finish(self, body, "media template slot");
        Some(TemplateSlotDecl::Media {
            accepts,
            fill,
            label,
            minimum_source_duration,
            span,
        })
    }
}

fn parse_accepts(
    parser: &mut Parser,
    value: crate::authoring::Identifier,
) -> Option<Spanned<TemplateMediaKindDecl>> {
    let parsed = match value.value.as_str() {
        "video" => TemplateMediaKindDecl::Video,
        "image" => TemplateMediaKindDecl::Image,
        "video-or-image" => TemplateMediaKindDecl::VideoOrImage,
        _ => {
            return invalid(
                parser,
                "accepts must be video, image, or video-or-image",
                value.span,
            )
        }
    };
    Some(Spanned {
        value: parsed,
        span: value.span,
    })
}

fn parse_fill(
    parser: &mut Parser,
    value: crate::authoring::Identifier,
) -> Option<Spanned<TemplateFillDecl>> {
    let parsed = match value.value.as_str() {
        "fit-duration" => TemplateFillDecl::FitDuration,
        "take-head" => TemplateFillDecl::TakeHead,
        "take-center" => TemplateFillDecl::TakeCenter,
        _ => {
            return invalid(
                parser,
                "fill must be fit-duration, take-head, or take-center",
                value.span,
            )
        }
    };
    Some(Spanned {
        value: parsed,
        span: value.span,
    })
}

fn invalid<T>(parser: &mut Parser, message: &str, span: Span) -> Option<T> {
    parser.error("AUTHORING_TEMPLATE_SLOT_VALUE", message.to_owned(), span);
    None
}
