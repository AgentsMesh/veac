use crate::authoring::{
    ColorMatrix, ColorPrimaries, ColorRange, ColorSpace, ColorTransfer, SemanticBlock,
    SemanticEntry, SemanticValue, VideoRateControl,
};

use super::output_fields::{field_enum, take_u64};
use super::semantic::finish;
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
            "bitrate" => self.bitrate(entry.block, entry.span),
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
        let value = take_u64(self, &mut block, "value")
            .and_then(|value| u8::try_from(value).ok())
            .unwrap_or(23);
        finish(self, block, "crf rate control");
        VideoRateControl::Crf { value }
    }

    fn bitrate(
        &mut self,
        block: Option<SemanticBlock>,
        span: crate::authoring::Span,
    ) -> VideoRateControl {
        let Some(mut block) = block else {
            self.error(
                "AUTHORING_OUTPUT_RATE_CONTROL",
                "bitrate requires a body".into(),
                span,
            );
            return VideoRateControl::Bitrate {
                target_bps: 8_000_000,
                max_bps: None,
                buffer_bps: None,
            };
        };
        let output = VideoRateControl::Bitrate {
            target_bps: take_u64(self, &mut block, "target-bps").unwrap_or(8_000_000),
            max_bps: take_u64(self, &mut block, "max-bps"),
            buffer_bps: take_u64(self, &mut block, "buffer-bps"),
        };
        finish(self, block, "bitrate rate control");
        output
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
