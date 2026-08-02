use crate::authoring::{
    ArtifactDecl, ArtifactRecipe, ArtifactTargetDecl, AudioMixSourceDecl, DeliveryDecl,
    DeliveryRasterDecl,
};
use veac_ir::{
    AudioMixSource, AudioStemOutput, CaptionSidecarOutput, Deliverable, DeliverableKind,
    DeliverableTarget, ImageSequenceOutput, RasterSettings, RenderConfig, ScopeOutput,
};

use super::{context::Context, delivery_extended, ids, output_types, settings, value};

pub(super) fn lower(ctx: &mut Context, value: &DeliveryDecl) -> Option<RenderConfig> {
    let mut deliverables = value
        .artifacts
        .iter()
        .map(|artifact| lower_artifact(ctx, artifact))
        .collect::<Option<Vec<_>>>()?;
    deliverables.sort_by(|left, right| left.id.cmp(&right.id));
    Some(RenderConfig {
        id: ids::render_config(ctx, &value.id)?,
        sequence_id: ids::sequence(ctx, &value.sequence)?,
        raster: match &value.raster {
            Some(raster) => Some(lower_raster(ctx, raster)?),
            None => None,
        },
        deliverables,
    })
}

fn lower_raster(ctx: &mut Context, value: &DeliveryRasterDecl) -> Option<RasterSettings> {
    Some(RasterSettings {
        width: settings::dimension(ctx, &value.width)?,
        height: settings::dimension(ctx, &value.height)?,
        frame_rate: settings::frame_rate(ctx, &value.frame_rate)?,
        captions: output_types::caption_output(value.captions),
    })
}

fn lower_artifact(ctx: &mut Context, value: &ArtifactDecl) -> Option<Deliverable> {
    let kind = match &value.recipe {
        ArtifactRecipe::Video(encoding) => DeliverableKind::Video(output_types::video(encoding)),
        ArtifactRecipe::ImageSequence(encoding) => {
            DeliverableKind::ImageSequence(ImageSequenceOutput {
                format: output_types::image_format(encoding.format),
                start_number: encoding.start_number,
            })
        }
        ArtifactRecipe::CaptionSidecar(encoding) => {
            DeliverableKind::CaptionSidecar(caption_sidecar(ctx, encoding)?)
        }
        ArtifactRecipe::AudioStem(encoding) => DeliverableKind::AudioStem(AudioStemOutput {
            format: output_types::stem_format(encoding.format),
            audio: output_types::audio(&encoding.audio),
            source: stem_source(ctx, &encoding.source)?,
        }),
        ArtifactRecipe::Scope(encoding) => DeliverableKind::Scope(ScopeOutput {
            scope: output_types::scope(encoding.scope),
            at: value::time(ctx, &encoding.at)?,
            width: encoding.width,
            height: encoding.height,
            format: output_types::image_format(encoding.format),
        }),
        ArtifactRecipe::AudioFile(encoding) => delivery_extended::audio_file(ctx, encoding)?,
        ArtifactRecipe::AnimatedImage(encoding) => delivery_extended::animated_image(encoding),
        ArtifactRecipe::StillImage(encoding) => delivery_extended::still_image(ctx, encoding)?,
        ArtifactRecipe::AdaptivePackage(encoding) => {
            delivery_extended::adaptive_package(ctx, encoding)?
        }
    };
    Some(Deliverable {
        id: ids::deliverable(ctx, &value.id)?,
        target: lower_target(&value.target),
        kind,
    })
}

fn caption_sidecar(
    ctx: &mut Context,
    value: &crate::authoring::CaptionSidecarEncoding,
) -> Option<CaptionSidecarOutput> {
    let mut track_ids = value
        .track_ids
        .iter()
        .map(|id| ids::track(ctx, id))
        .collect::<Option<Vec<_>>>()?;
    track_ids.sort();
    Some(CaptionSidecarOutput {
        format: output_types::caption_format(value.format),
        track_ids,
    })
}

fn lower_target(value: &ArtifactTargetDecl) -> DeliverableTarget {
    match value {
        ArtifactTargetDecl::File(value) => DeliverableTarget::File {
            name: value.value.clone(),
        },
        ArtifactTargetDecl::ImageSequence(value) => DeliverableTarget::ImageSequence {
            pattern: value.value.clone(),
        },
        ArtifactTargetDecl::Package(value) => DeliverableTarget::Package {
            name: value.value.clone(),
        },
    }
}

pub(super) fn stem_source(ctx: &mut Context, value: &AudioMixSourceDecl) -> Option<AudioMixSource> {
    Some(match value {
        AudioMixSourceDecl::Master => AudioMixSource::Master,
        AudioMixSourceDecl::Track(id) => AudioMixSource::Track {
            track_id: ids::track(ctx, id)?,
        },
        AudioMixSourceDecl::Bus(id) => AudioMixSource::Bus {
            bus_id: ids::bus_id(ctx, id)?,
        },
    })
}
