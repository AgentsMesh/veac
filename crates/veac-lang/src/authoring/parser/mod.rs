mod annotation;
mod annotation_fields;
mod annotation_payload;
mod annotation_payload_analysis;
mod annotation_target;
mod audio_processor;
mod color_space;
mod color_stage;
mod delivery_raster;
mod entry;
mod generator;
mod generator_geometry;
mod generator_gradient;
mod generator_shape;
mod interpolation;
mod item;
mod mapping;
mod mapping_policy;
mod modifier;
mod modifier_audio;
mod modifier_color;
mod modifier_effect;
mod modifier_mask;
mod modifier_static;
mod modifier_surface;
mod multicam;
mod output_aux;
mod output_delivery;
mod output_fields;
mod output_video;
mod output_video_rate;
mod parameter;
mod project;
mod raw;
mod relation;
mod relation_endpoint;
mod relation_matte;
mod relation_membership;
mod relation_sidechain;
mod relation_style;
mod relation_style_motion;
mod relation_transition;
mod relation_value;
mod resource;
mod resource_fields;
mod semantic;
mod settings;
mod shadow;
mod source;
mod structure_apply;
mod structure_apply_mix;
mod structure_apply_scope;
mod structure_delivery;
mod template_slot;
mod text_animation;
mod text_decor;
mod text_layout;
mod text_source;
mod text_span;
mod text_style;
mod text_value;
mod timeline;
mod timeline_state;
mod validate;
mod validate_annotation;
mod validate_delivery;
mod validate_delivery_extended;
mod validate_mapping;
mod validate_multicam;
mod validate_numbers;
mod validate_refs;
mod validate_source;
mod validate_template_slot;
mod value;

use super::lexer::{Token, TokenKind};
use super::{Diagnostic, Span};

pub use entry::parse;
#[cfg(test)]
pub(super) use entry::parse_with_limits;

pub(super) struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    pub(super) fn current(&self) -> &Token {
        &self.tokens[self.cursor]
    }

    pub(super) fn previous_span(&self) -> Span {
        self.tokens[self.cursor.saturating_sub(1)].span
    }

    pub(super) fn at_eof(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
            || crate::authoring::diagnostic_budget::exhausted(&self.diagnostics)
    }

    pub(super) fn at_word(&self, expected: &str) -> bool {
        matches!(&self.current().kind, TokenKind::Word(value) if value == expected)
    }

    pub(super) fn at_left_brace(&self) -> bool {
        matches!(self.current().kind, TokenKind::LeftBrace)
    }

    pub(super) fn at_right_brace(&self) -> bool {
        matches!(self.current().kind, TokenKind::RightBrace)
    }

    pub(super) fn at_semicolon(&self) -> bool {
        matches!(self.current().kind, TokenKind::Semicolon)
    }

    pub(super) fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if !self.at_eof() {
            self.cursor += 1;
        }
        token
    }

    pub(super) fn required_word(&mut self, expected: &'static str) -> Option<Span> {
        if self.at_word(expected) {
            Some(self.advance().span)
        } else {
            self.expected(format!("`{expected}`"));
            None
        }
    }

    pub(super) fn left_brace(&mut self) -> Option<Span> {
        self.punctuation(TokenKind::LeftBrace, "`{`")
    }

    pub(super) fn right_brace(&mut self) -> Option<Span> {
        self.punctuation(TokenKind::RightBrace, "`}`")
    }

    pub(super) fn semicolon(&mut self) -> Option<Span> {
        self.punctuation(TokenKind::Semicolon, "`;`")
    }

    fn punctuation(&mut self, expected: TokenKind, label: &'static str) -> Option<Span> {
        if std::mem::discriminant(&self.current().kind) == std::mem::discriminant(&expected) {
            Some(self.advance().span)
        } else {
            self.expected(label.to_owned());
            None
        }
    }

    pub(super) fn error(&mut self, code: &'static str, message: String, span: Span) {
        crate::authoring::diagnostic_budget::push(
            &mut self.diagnostics,
            Diagnostic {
                code,
                message,
                span,
            },
        );
    }

    pub(super) fn diagnostic(&mut self, diagnostic: Diagnostic) {
        crate::authoring::diagnostic_budget::push(&mut self.diagnostics, diagnostic);
    }

    pub(super) fn duplicate(&mut self, field: &'static str, span: Span) {
        self.error(
            "AUTHORING_DUPLICATE_FIELD",
            format!("{field} is declared more than once"),
            span,
        );
    }

    fn expected(&mut self, expected: String) {
        self.error(
            "AUTHORING_EXPECTED_TOKEN",
            format!("expected {expected}"),
            self.current().span,
        );
    }
}
