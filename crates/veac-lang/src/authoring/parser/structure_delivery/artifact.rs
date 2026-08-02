use crate::authoring::{
    ArtifactDecl, ArtifactRecipe, ArtifactTargetDecl, Identifier, SemanticBlock, SemanticEntry,
    SemanticValue,
};

use super::super::output_fields::is_leaf_name;
use super::super::semantic::{finish, required, take};
use super::super::Parser;

impl Parser {
    pub(super) fn artifact(&mut self) -> Option<ArtifactDecl> {
        let start = self.required_word("artifact")?;
        let kind = self.identifier("artifact kind")?;
        let id = self.identifier("artifact")?;
        let mut body = self.semantic_block()?;
        if let Some(legacy) = take(self, &mut body, "encoding") {
            self.error(
                "AUTHORING_LEGACY_ARTIFACT_ENCODING",
                "`encoding {}` was removed; use typed artifact primitives".into(),
                legacy.span,
            );
        }
        let target = required(self, &mut body, "target", "artifact")
            .and_then(|entry| self.artifact_target(&entry))?;
        let recipe = self.artifact_recipe(&kind, &mut body)?;
        let span = start.join(body.span);
        finish(self, body, "artifact");
        Some(ArtifactDecl {
            id,
            target,
            recipe,
            span,
        })
    }

    fn artifact_recipe(
        &mut self,
        kind: &Identifier,
        body: &mut SemanticBlock,
    ) -> Option<ArtifactRecipe> {
        Some(match kind.value.as_str() {
            "video" => ArtifactRecipe::Video(self.video_recipe(body)?),
            "image-sequence" => ArtifactRecipe::ImageSequence(self.image_sequence_recipe(body)?),
            "caption-sidecar" => ArtifactRecipe::CaptionSidecar(self.caption_sidecar_recipe(body)?),
            "audio-stem" => ArtifactRecipe::AudioStem(self.audio_stem_recipe(body)?),
            "scope" => ArtifactRecipe::Scope(self.scope_recipe(body)?),
            "audio-file" => ArtifactRecipe::AudioFile(self.audio_file_recipe(body)?),
            "animated-image" => ArtifactRecipe::AnimatedImage(self.animated_image_recipe(body)?),
            "still-image" => ArtifactRecipe::StillImage(self.still_image_recipe(body)?),
            "adaptive-package" => {
                ArtifactRecipe::AdaptivePackage(self.adaptive_package_recipe(body)?)
            }
            other => {
                self.error(
                    "AUTHORING_ARTIFACT_KIND",
                    format!("unsupported artifact kind '{other}'"),
                    kind.span,
                );
                return None;
            }
        })
    }

    fn artifact_target(&mut self, entry: &SemanticEntry) -> Option<ArtifactTargetDecl> {
        let parsed = match entry.values.as_slice() {
            [SemanticValue::Identifier(kind), SemanticValue::String(value)]
                if entry.block.is_none() =>
            {
                match kind.value.as_str() {
                    "file" => Some(ArtifactTargetDecl::File(value.clone())),
                    "pattern" => Some(ArtifactTargetDecl::ImageSequence(value.clone())),
                    "package" => Some(ArtifactTargetDecl::Package(value.clone())),
                    _ => None,
                }
            }
            _ => None,
        };
        let Some(target) = parsed else {
            self.error(
                "AUTHORING_ARTIFACT_TARGET",
                "target must be file, pattern, or package with one safe name".into(),
                entry.span,
            );
            return None;
        };
        self.validate_target_name(&target);
        Some(target)
    }

    fn validate_target_name(&mut self, target: &ArtifactTargetDecl) {
        let leaf = matches!(
            target,
            ArtifactTargetDecl::File(value) | ArtifactTargetDecl::Package(value)
                if is_leaf_name(&value.value)
        );
        if matches!(
            target,
            ArtifactTargetDecl::File(_) | ArtifactTargetDecl::Package(_)
        ) && !leaf
        {
            let package = matches!(target, ArtifactTargetDecl::Package(_));
            self.error(
                if package {
                    "AUTHORING_ARTIFACT_TARGET_PACKAGE"
                } else {
                    "AUTHORING_ARTIFACT_TARGET_FILE"
                },
                "file and package targets require a safe leaf basename".into(),
                target.value().span,
            );
        }
    }
}
