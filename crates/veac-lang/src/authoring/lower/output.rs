use crate::authoring::{AudioStemSourceDecl, OutputDecl, OutputEncoding};
use veac_ir::{
    AudioStemOutput, AudioStemSource, CaptionSidecarOutput, Deliverable, DeliverableKind,
    ImageSequenceOutput, RenderConfig, ScopeOutput,
};

use super::{context::Context, ids, output_types, settings::LoweredSettings, value};

pub(super) fn lower(
    ctx: &mut Context,
    declaration: &OutputDecl,
    settings: &LoweredSettings,
) -> Option<RenderConfig> {
    let kind = match &declaration.encoding {
        OutputEncoding::Video(encoding) => DeliverableKind::Video(output_types::video(encoding)),
        OutputEncoding::ImageSequence(encoding) => {
            DeliverableKind::ImageSequence(ImageSequenceOutput {
                format: output_types::image_format(encoding.format),
                start_number: encoding.start_number,
            })
        }
        OutputEncoding::CaptionSidecar(encoding) => {
            let mut track_ids = encoding
                .track_ids
                .iter()
                .map(|id| ids::track(ctx, id))
                .collect::<Option<Vec<_>>>()?;
            track_ids.sort();
            DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
                format: output_types::caption_format(encoding.format),
                track_ids,
            })
        }
        OutputEncoding::AudioStem(encoding) => DeliverableKind::AudioStem(AudioStemOutput {
            format: output_types::stem_format(encoding.format),
            audio: output_types::audio(&encoding.audio),
            source: stem_source(ctx, &encoding.source)?,
        }),
        OutputEncoding::Scope(encoding) => DeliverableKind::Scope(ScopeOutput {
            scope: output_types::scope(encoding.scope),
            at: value::time(ctx, &encoding.at)?,
            width: encoding.width,
            height: encoding.height,
            format: output_types::image_format(encoding.format),
        }),
    };
    Some(RenderConfig {
        id: ids::render_config(ctx, &declaration.id)?,
        sequence_id: ids::sequence(ctx, &declaration.sequence)?,
        width: settings.width,
        height: settings.height,
        frame_rate: settings.frame_rate,
        deliverables: vec![Deliverable {
            id: ids::deliverable(ctx, &declaration.id)?,
            file_name: declaration.file_name.value.clone(),
            kind,
        }],
    })
}

fn stem_source(ctx: &mut Context, value: &AudioStemSourceDecl) -> Option<AudioStemSource> {
    Some(match value {
        AudioStemSourceDecl::Master => AudioStemSource::Master,
        AudioStemSourceDecl::Track(id) => AudioStemSource::Track {
            track_id: ids::track(ctx, id)?,
        },
        AudioStemSourceDecl::Bus(id) => AudioStemSource::Bus {
            bus_id: ids::bus_id(ctx, id)?,
        },
    })
}
