use super::{generator_gradient, generator_shape, Parser};
use crate::authoring::{GeneratorDecl, Span, Spanned};

impl Parser {
    pub(super) fn generator(&mut self) -> Option<GeneratorDecl> {
        let kind = self.identifier("generator kind")?;
        match kind.value.as_str() {
            "transparent" => {
                let end = self.semicolon()?;
                Some(GeneratorDecl::Transparent(kind.span.join(end)))
            }
            "silence" => {
                let end = self.semicolon()?;
                Some(GeneratorDecl::Silence(kind.span.join(end)))
            }
            "solid" => self.solid_generator(kind.span),
            "gradient" => generator_gradient::parse(self, kind.span).map(GeneratorDecl::Gradient),
            "shape" => generator_shape::parse(self, kind.span).map(GeneratorDecl::Shape),
            _ => {
                self.error(
                    "AUTHORING_GENERATOR_KIND",
                    format!("unknown generator kind `{}`", kind.value),
                    kind.span,
                );
                self.recover_declaration();
                None
            }
        }
    }

    fn solid_generator(&mut self, start: Span) -> Option<GeneratorDecl> {
        self.left_brace()?;
        let mut color = None;
        while !self.at_right_brace() && !self.at_eof() {
            let field = self.identifier("solid generator field")?;
            if field.value == "color" {
                let value = self.color("solid generator");
                self.semicolon();
                assign(self, "solid color", &mut color, value, field.span);
            } else {
                self.generator_field_error(&field.value, field.span);
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        let color = required(self, "color", color, span)?;
        Some(GeneratorDecl::Solid { color, span })
    }

    fn generator_field_error(&mut self, field: &str, span: Span) {
        self.error(
            "AUTHORING_GENERATOR_FIELD",
            format!("unknown generator field `{field}`"),
            span,
        );
        self.recover_declaration();
    }
}

pub(super) fn assign<T>(
    parser: &mut Parser,
    name: &'static str,
    slot: &mut Option<T>,
    value: Option<T>,
    span: Span,
) {
    if slot.is_some() {
        parser.duplicate(name, span);
    } else if let Some(value) = value {
        *slot = Some(value);
    }
}

pub(super) fn required<T>(
    parser: &mut Parser,
    name: &'static str,
    value: Option<T>,
    span: Span,
) -> Option<T> {
    if value.is_none() {
        parser.error(
            "AUTHORING_GENERATOR_REQUIRED",
            format!("generator requires `{name}`"),
            span,
        );
    }
    value
}

pub(super) fn paint_span(value: &crate::authoring::PaintDecl) -> Span {
    match value {
        crate::authoring::PaintDecl::Solid(Spanned { span, .. }) => *span,
        crate::authoring::PaintDecl::Gradient(value) => value.span,
    }
}
