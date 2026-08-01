use crate::authoring::{
    HlsCappedBitrateDecl, HlsH264EncodingDecl, HlsH264ProfileDecl, HlsRenditionDecl, SemanticBlock,
    SemanticEntry, SemanticValue,
};

use super::super::super::output_fields::tagged_block;
use super::super::super::semantic::{finish, required, take, word};
use super::super::super::Parser;

impl Parser {
    pub(super) fn hls_renditions(&mut self, package: &mut SemanticBlock) -> Vec<HlsRenditionDecl> {
        let entries = std::mem::take(&mut package.entries);
        let mut values = Vec::new();
        for entry in entries {
            if entry.name.value == "rendition" {
                if let Some(value) = self.hls_rendition(entry) {
                    values.push(value);
                }
            } else {
                package.entries.push(entry);
            }
        }
        values
    }

    fn hls_rendition(&mut self, entry: SemanticEntry) -> Option<HlsRenditionDecl> {
        let ([SemanticValue::Identifier(id)], Some(mut block)) =
            (entry.values.as_slice(), entry.block)
        else {
            self.error(
                "AUTHORING_HLS_RENDITION",
                "rendition requires a stable identifier and body".into(),
                entry.span,
            );
            return None;
        };
        let canvas = required(self, &mut block, "canvas", "HLS rendition")?;
        let (width, height) = self.rendition_canvas(&canvas)?;
        let encode = required(self, &mut block, "encode", "HLS rendition")?;
        let (codec, settings) = tagged_block(self, encode, "HLS rendition encode")?;
        if codec.value != "h264" {
            return self.invalid_recipe("HLS rendition", &codec);
        }
        let encoding = self.hls_h264(settings)?;
        finish(self, block, "HLS rendition");
        Some(HlsRenditionDecl {
            id: id.clone(),
            width,
            height,
            encoding,
        })
    }

    fn rendition_canvas(
        &mut self,
        entry: &SemanticEntry,
    ) -> Option<(
        crate::authoring::NumberLiteral,
        crate::authoring::NumberLiteral,
    )> {
        match entry.values.as_slice() {
            [SemanticValue::Number(width), SemanticValue::Identifier(by), SemanticValue::Number(height)]
                if by.value == "by" && entry.block.is_none() =>
            {
                Some((width.clone(), height.clone()))
            }
            _ => {
                self.error(
                    "AUTHORING_HLS_CANVAS",
                    "rendition canvas must be `<width>px by <height>px`".into(),
                    entry.span,
                );
                None
            }
        }
    }

    fn hls_h264(&mut self, mut block: SemanticBlock) -> Option<HlsH264EncodingDecl> {
        let entry = required(self, &mut block, "rate-control", "HLS H264 encode")?;
        let (mode, mut rate) = tagged_block(self, entry, "HLS rate-control")?;
        if mode.value != "capped" {
            return self.invalid_recipe("HLS rate-control", &mode);
        }
        let rate_control = HlsCappedBitrateDecl {
            target: super::super::required_number(
                self,
                &mut rate,
                "target",
                "capped rate-control",
            )?,
            max: super::super::required_number(self, &mut rate, "max", "capped rate-control")?,
            buffer: super::super::required_number(
                self,
                &mut rate,
                "buffer",
                "capped rate-control",
            )?,
        };
        finish(self, rate, "capped rate-control");
        let profile = take(self, &mut block, "profile")
            .and_then(|entry| word(self, &entry, "HLS profile"))
            .and_then(|value| self.hls_profile(&value));
        let level = take(self, &mut block, "level").and_then(|entry| self.hls_level(entry));
        let color_space = self.hls_color_space(&mut block);
        let b_frames = self.hls_b_frames(&mut block);
        finish(self, block, "HLS H264 encode");
        Some(HlsH264EncodingDecl {
            rate_control,
            profile,
            level,
            color_space,
            b_frames,
        })
    }

    fn hls_profile(&mut self, value: &crate::authoring::Identifier) -> Option<HlsH264ProfileDecl> {
        match value.value.as_str() {
            "baseline" => Some(HlsH264ProfileDecl::Baseline),
            "main" => Some(HlsH264ProfileDecl::Main),
            "high" => Some(HlsH264ProfileDecl::High),
            "automatic" => None,
            _ => self.invalid_recipe("HLS H264 profile", value),
        }
    }

    fn hls_level(&mut self, entry: SemanticEntry) -> Option<String> {
        match (entry.values.as_slice(), entry.block) {
            ([SemanticValue::Identifier(value)], None) if value.value == "automatic" => None,
            ([SemanticValue::String(value)], None) => Some(value.value.clone()),
            _ => {
                self.error(
                    "AUTHORING_HLS_LEVEL",
                    "HLS level must be automatic or one string".into(),
                    entry.span,
                );
                None
            }
        }
    }

    fn hls_color_space(
        &mut self,
        block: &mut SemanticBlock,
    ) -> Option<crate::authoring::ColorSpace> {
        let entry = take(self, block, "color-space")?;
        match (entry.values.as_slice(), entry.block) {
            ([SemanticValue::Identifier(value)], None) if value.value == "source" => None,
            ([], Some(block)) => Some(self.color_space(block)),
            _ => {
                self.error(
                    "AUTHORING_HLS_COLOR_SPACE",
                    "color-space must be `source` or a color-space block".into(),
                    entry.span,
                );
                None
            }
        }
    }

    fn hls_b_frames(&mut self, block: &mut SemanticBlock) -> Option<u8> {
        let entry = take(self, block, "b-frames")?;
        match entry.values.as_slice() {
            [SemanticValue::Identifier(value)]
                if value.value == "automatic" && entry.block.is_none() =>
            {
                None
            }
            [SemanticValue::Number(value)] if entry.block.is_none() => match value.raw.parse() {
                Ok(parsed) => Some(parsed),
                Err(_) => {
                    self.error(
                        "AUTHORING_HLS_B_FRAMES",
                        "b-frames must be automatic or an unsigned 8-bit integer".into(),
                        entry.span,
                    );
                    None
                }
            },
            _ => {
                self.error(
                    "AUTHORING_HLS_B_FRAMES",
                    "b-frames must be automatic or an unsigned 8-bit integer".into(),
                    entry.span,
                );
                None
            }
        }
    }
}
