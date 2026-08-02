use crate::authoring::{
    ColorMatrix, ColorPrimaries, ColorRange, ColorSpace, ColorTransfer, SemanticBlock,
    SemanticEntry, SemanticValue, VideoRateControl,
};

use super::output_fields::{bitrate, buffer_size, field_enum};
use super::semantic::{finish, number, required};
use super::Parser;

impl Parser {
    pub(super) fn rate_control(&mut self, entry: SemanticEntry) -> VideoRateControl {
        let Some(SemanticValue::Identifier(kind)) = entry.values.first() else {
            self.error(
                "AUTHORING_OUTPUT_RATE_CONTROL",
                "rate-control requires a variant".into(),
                entry.span,
            );
            return VideoRateControl::Crf { value: 23 };
        };
        if entry.values.len() != 1 {
            self.error(
                "AUTHORING_OUTPUT_RATE_CONTROL",
                "rate-control accepts one variant".into(),
                entry.span,
            );
        }
        match kind.value.as_str() {
            "crf" => self.crf(entry.block, entry.span),
            "average" => self.average(entry.block, entry.span),
            "capped" => self.capped(entry.block, entry.span),
            "lossless" if entry.block.is_none() => VideoRateControl::Lossless,
            "lossless" => {
                self.error(
                    "AUTHORING_OUTPUT_RATE_CONTROL",
                    "lossless does not accept a body".into(),
                    entry.span,
                );
                VideoRateControl::Lossless
            }
            other => {
                self.error(
                    "AUTHORING_OUTPUT_ENUM",
                    format!("invalid rate-control variant '{other}'"),
                    kind.span,
                );
                VideoRateControl::Crf { value: 23 }
            }
        }
    }

    fn crf(
        &mut self,
        block: Option<SemanticBlock>,
        span: crate::authoring::Span,
    ) -> VideoRateControl {
        let Some(mut block) = block else {
            self.error(
                "AUTHORING_OUTPUT_RATE_CONTROL",
                "crf requires a body".into(),
                span,
            );
            return VideoRateControl::Crf { value: 23 };
        };
        let value = required(self, &mut block, "value", "CRF rate-control")
            .and_then(|entry| number(self, &entry, "CRF value"))
            .and_then(|value| value.raw.parse::<u8>().ok())
            .unwrap_or(23);
        finish(self, block, "crf rate control");
        VideoRateControl::Crf { value }
    }

    fn average(
        &mut self,
        block: Option<SemanticBlock>,
        span: crate::authoring::Span,
    ) -> VideoRateControl {
        let Some(mut block) = block else {
            self.error(
                "AUTHORING_OUTPUT_RATE_CONTROL",
                "average rate-control requires a body".into(),
                span,
            );
            return VideoRateControl::Bitrate {
                target_bps: 8_000_000,
                max_bps: None,
                buffer_size_bits: None,
            };
        };
        let target = required(self, &mut block, "target", "average rate-control")
            .and_then(|entry| number(self, &entry, "average target"))
            .and_then(|value| bitrate(self, &value, "average target"))
            .unwrap_or(8_000_000);
        finish(self, block, "average rate-control");
        VideoRateControl::Bitrate {
            target_bps: target,
            max_bps: None,
            buffer_size_bits: None,
        }
    }

    fn capped(
        &mut self,
        block: Option<SemanticBlock>,
        span: crate::authoring::Span,
    ) -> VideoRateControl {
        let Some(mut block) = block else {
            self.error(
                "AUTHORING_OUTPUT_RATE_CONTROL",
                "capped rate-control requires a body".into(),
                span,
            );
            return VideoRateControl::Bitrate {
                target_bps: 8_000_000,
                max_bps: Some(8_560_000),
                buffer_size_bits: Some(16_000_000),
            };
        };
        let target = required_rate(self, &mut block, "target").unwrap_or(8_000_000);
        let max = required_rate(self, &mut block, "max").unwrap_or(8_560_000);
        let buffer = required(self, &mut block, "buffer", "capped rate-control")
            .and_then(|entry| number(self, &entry, "rate-control buffer"))
            .and_then(|value| buffer_size(self, &value, "rate-control buffer"))
            .unwrap_or(16_000_000);
        finish(self, block, "capped rate-control");
        VideoRateControl::Bitrate {
            target_bps: target,
            max_bps: Some(max),
            buffer_size_bits: Some(buffer),
        }
    }

    pub(super) fn color_space(&mut self, mut block: SemanticBlock) -> ColorSpace {
        let mut output = ColorSpace {
            primaries: ColorPrimaries::Bt709,
            transfer: ColorTransfer::Bt709,
            matrix: ColorMatrix::Bt709,
            range: ColorRange::Limited,
        };
        field_enum(self, &mut block, "primaries", &mut output.primaries);
        field_enum(self, &mut block, "transfer", &mut output.transfer);
        field_enum(self, &mut block, "matrix", &mut output.matrix);
        field_enum(self, &mut block, "range", &mut output.range);
        finish(self, block, "color space");
        output
    }
}

fn required_rate(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
) -> Option<u64> {
    required(parser, block, name, "capped rate-control")
        .and_then(|entry| number(parser, &entry, name))
        .and_then(|value| bitrate(parser, &value, name))
}
