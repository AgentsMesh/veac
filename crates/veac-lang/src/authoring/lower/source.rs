use crate::authoring::SourceDecl;
use veac_ir::ClipSource;

use super::{context::Context, generator, ids, multicam};

pub fn lower(ctx: &mut Context, value: &SourceDecl) -> Option<ClipSource> {
    match value {
        SourceDecl::Media { resource, .. } => Some(ClipSource::Media {
            material_id: ids::material(ctx, &resource.id)?,
        }),
        SourceDecl::Text { text, .. } => super::text::lower(ctx, text),
        SourceDecl::Caption { caption, .. } => super::text::caption(ctx, caption),
        SourceDecl::Sequence { sequence, .. } => Some(ClipSource::Sequence {
            sequence_id: ids::sequence(ctx, &sequence.id)?,
        }),
        SourceDecl::Generated { generator, .. } => Some(ClipSource::Generated {
            generator: generator::lower(ctx, generator)?,
        }),
        SourceDecl::Multicam {
            group, switches, ..
        } => multicam::source(ctx, group, switches),
    }
}
