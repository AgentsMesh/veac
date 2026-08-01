use crate::authoring::{
    ArtifactDecl, ArtifactRecipe, ArtifactTargetDecl, DeliveryDecl, OutputKeyword,
};

use super::output_aux::{audio_stem, caption_sidecar, image_sequence, scope};
use super::output_video::video;
use super::value::quoted;
use super::writer::Writer;

pub(super) fn delivery(writer: &mut Writer, value: &DeliveryDecl) {
    writer.block(format!("delivery {}", value.id.value), |writer| {
        writer.line(format!("sequence {};", value.sequence.value));
        if let Some(raster) = &value.raster {
            writer.block("raster", |writer| {
                writer.line(format!(
                    "canvas {} by {};",
                    raster.width.raw, raster.height.raw
                ));
                writer.line(format!("frame-rate {};", raster.frame_rate.raw));
                writer.line(format!("captions {};", raster.captions.token()));
            });
        }
        for artifact in &value.artifacts {
            writer.blank();
            artifact_block(writer, artifact);
        }
    });
}

fn artifact_block(writer: &mut Writer, value: &ArtifactDecl) {
    writer.block(
        format!("artifact {} {}", value.recipe.kind(), value.id.value),
        |writer| {
            writer.line(format!(
                "target {} {};",
                target_kind(&value.target),
                quoted(&value.target.value().value)
            ));
            recipe(writer, &value.recipe);
        },
    );
}

fn target_kind(value: &ArtifactTargetDecl) -> &'static str {
    match value {
        ArtifactTargetDecl::File(_) => "file",
        ArtifactTargetDecl::ImageSequence(_) => "pattern",
        ArtifactTargetDecl::Package(_) => "package",
    }
}

fn recipe(writer: &mut Writer, value: &ArtifactRecipe) {
    match value {
        ArtifactRecipe::Video(value) => video(writer, value),
        ArtifactRecipe::ImageSequence(value) => image_sequence(writer, value),
        ArtifactRecipe::CaptionSidecar(value) => caption_sidecar(writer, value),
        ArtifactRecipe::AudioStem(value) => audio_stem(writer, value),
        ArtifactRecipe::Scope(value) => scope(writer, value),
        ArtifactRecipe::AudioFile(value) => super::delivery_extended::audio_file(writer, value),
        ArtifactRecipe::AnimatedImage(value) => {
            super::delivery_extended::animated_image(writer, value)
        }
        ArtifactRecipe::StillImage(value) => super::delivery_extended::still_image(writer, value),
        ArtifactRecipe::AdaptivePackage(value) => {
            super::delivery_extended::adaptive_package(writer, value)
        }
    }
}
