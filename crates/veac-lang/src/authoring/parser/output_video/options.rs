use crate::authoring::{OutputKeyword, SemanticBlock, SemanticEntry, SemanticValue, VideoProfile};

use super::super::semantic::{take, word};
use super::super::Parser;

impl Parser {
    pub(in crate::authoring::parser) fn optional_u32(
        &mut self,
        block: &mut SemanticBlock,
        name: &'static str,
        fallback: Option<u32>,
    ) -> Option<u32> {
        let Some(entry) = take(self, block, name) else {
            return fallback;
        };
        self.optional_unsigned(entry, name, fallback)
    }

    pub(in crate::authoring::parser) fn optional_u8(
        &mut self,
        block: &mut SemanticBlock,
        name: &'static str,
        fallback: Option<u8>,
    ) -> Option<u8> {
        let Some(entry) = take(self, block, name) else {
            return fallback;
        };
        self.optional_unsigned(entry, name, fallback)
    }

    fn optional_unsigned<T: std::str::FromStr + Copy>(
        &mut self,
        entry: SemanticEntry,
        name: &'static str,
        fallback: Option<T>,
    ) -> Option<T> {
        match entry.values.as_slice() {
            [SemanticValue::Identifier(value)]
                if value.value == "automatic" && entry.block.is_none() =>
            {
                None
            }
            [SemanticValue::Number(value)] if entry.block.is_none() => match value.raw.parse() {
                Ok(parsed) => Some(parsed),
                Err(_) => {
                    self.integer_error(name, entry.span);
                    fallback
                }
            },
            _ => {
                self.integer_error(name, entry.span);
                fallback
            }
        }
    }

    fn integer_error(&mut self, name: &'static str, span: crate::authoring::Span) {
        self.error(
            "AUTHORING_OUTPUT_INTEGER",
            format!("{name} must be automatic or an unsigned integer in range"),
            span,
        );
    }

    pub(in crate::authoring::parser) fn optional_profile(
        &mut self,
        entry: SemanticEntry,
        fallback: Option<VideoProfile>,
    ) -> Option<VideoProfile> {
        match word(self, &entry, "video profile") {
            Some(value) if value.value == "automatic" => None,
            Some(value) => VideoProfile::parse(&value.value).or_else(|| {
                self.error(
                    "AUTHORING_OUTPUT_ENUM",
                    format!("invalid video profile '{}'", value.value),
                    value.span,
                );
                fallback
            }),
            None => fallback,
        }
    }

    pub(in crate::authoring::parser) fn optional_level(
        &mut self,
        entry: SemanticEntry,
        fallback: Option<String>,
    ) -> Option<String> {
        match (entry.values.as_slice(), entry.block) {
            ([SemanticValue::Identifier(value)], None) if value.value == "automatic" => None,
            ([SemanticValue::String(value)], None) => Some(value.value.clone()),
            _ => {
                self.error(
                    "AUTHORING_OUTPUT_LEVEL",
                    "level must be automatic or one string".into(),
                    entry.span,
                );
                fallback
            }
        }
    }
}
