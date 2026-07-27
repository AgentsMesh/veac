use std::collections::HashSet;

use crate::authoring::{GeneratorDecl, LayerKind, MappingDecl, SequenceDecl, SourceDecl};

use super::context::Context;

pub(super) fn derive(ctx: &Context, sequences: &[SequenceDecl]) -> HashSet<String> {
    let mut audible = HashSet::new();
    for _ in 0..sequences.len() {
        let before = audible.len();
        for sequence in sequences {
            if sequence.layers.iter().any(|layer| {
                matches!(layer.kind, LayerKind::Video | LayerKind::Audio)
                    && layer.items.iter().any(|item| {
                        !matches!(item.mapping, Some(MappingDecl::Freeze { .. }))
                            && source_is_audible(ctx, &audible, &item.source)
                    })
            }) {
                audible.insert(sequence.id.value.clone());
            }
        }
        if audible.len() == before {
            break;
        }
    }
    audible
}

fn source_is_audible(ctx: &Context, audible: &HashSet<String>, source: &SourceDecl) -> bool {
    match source {
        SourceDecl::Media { resource, .. } => ctx.audio_resources.contains(&resource.id.value),
        SourceDecl::Sequence { sequence, .. } => audible.contains(&sequence.id.value),
        SourceDecl::Generated {
            generator: GeneratorDecl::Silence { .. },
            ..
        }
        | SourceDecl::Multicam { .. } => true,
        _ => false,
    }
}
