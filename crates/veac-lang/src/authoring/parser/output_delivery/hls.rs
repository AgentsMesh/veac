mod rendition;

use crate::authoring::{
    AdaptivePackageRecipe, HlsAudioDecl, SemanticBlock, SemanticEntry, SemanticValue,
};

use super::super::output_fields::tagged_block;
use super::super::semantic::{finish, required};
use super::super::Parser;

impl Parser {
    pub(in crate::authoring::parser) fn adaptive_package_recipe(
        &mut self,
        body: &mut SemanticBlock,
    ) -> Option<AdaptivePackageRecipe> {
        let entry = required(self, body, "package", "adaptive-package artifact")?;
        let (format, mut package) = tagged_block(self, entry, "adaptive package")?;
        if format.value != "hls" {
            return self.invalid_recipe("adaptive-package", &format);
        }
        let segment_duration =
            super::required_number(self, &mut package, "segment-duration", "HLS package")?;
        let audio = required(self, &mut package, "audio", "HLS package")
            .and_then(|entry| self.hls_audio(entry))?;
        let renditions = self.hls_renditions(&mut package);
        if renditions.is_empty() {
            self.error(
                "AUTHORING_HLS_RENDITION_REQUIRED",
                "HLS package requires at least one named rendition".into(),
                package.span,
            );
        }
        finish(self, package, "HLS package");
        Some(AdaptivePackageRecipe {
            segment_duration,
            audio,
            renditions,
        })
    }

    fn hls_audio(&mut self, entry: SemanticEntry) -> Option<Option<HlsAudioDecl>> {
        match (entry.values.as_slice(), entry.block) {
            ([SemanticValue::Identifier(value)], None) if value.value == "none" => Some(None),
            ([], Some(mut block)) => {
                let source = required(self, &mut block, "source", "HLS audio")
                    .map(|entry| self.stem_source(entry))?;
                let encode = required(self, &mut block, "encode", "HLS audio")?;
                let (codec, mut settings) = tagged_block(self, encode, "HLS audio encode")?;
                if codec.value != "aac" {
                    return self.invalid_recipe("HLS audio", &codec);
                }
                let bitrate = super::required_number(self, &mut settings, "bitrate", "AAC encode")?;
                let sample_rate =
                    super::required_number(self, &mut settings, "sample-rate", "AAC encode")?;
                let layout =
                    super::required_word(self, &mut settings, "channel-layout", "AAC encode")?;
                let channel_layout = self.channel_layout(&layout)?;
                finish(self, settings, "AAC encode");
                finish(self, block, "HLS audio");
                Some(Some(HlsAudioDecl {
                    source,
                    bitrate,
                    sample_rate,
                    channel_layout,
                }))
            }
            _ => {
                self.error(
                    "AUTHORING_HLS_AUDIO",
                    "HLS audio must be `none` or an audio recipe block".into(),
                    entry.span,
                );
                None
            }
        }
    }
}
