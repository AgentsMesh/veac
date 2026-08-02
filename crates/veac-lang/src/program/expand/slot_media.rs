use std::collections::BTreeMap;

use crate::authoring::{Document, ResourceKind, Span};
use crate::program::diagnostic::Diagnostic;
use crate::program::lexer;
use crate::program::model::{ComponentCatalog, RawBlock, Scope, SlotKind, SurfaceFile};
use crate::program::token::Token;

pub(crate) fn validate(
    file: &SurfaceFile,
    scope: &Scope,
    catalog: &ComponentCatalog,
    document: &Document,
) -> Result<(), Diagnostic> {
    let resources = document
        .project
        .resources
        .iter()
        .map(|resource| (resource.id.value.as_str(), resource.kind))
        .collect::<BTreeMap<_, _>>();
    for instance in &file.instances {
        let Some(key) = scope.components.get(&instance.component) else {
            continue;
        };
        let Some(component) = catalog.get(key) else {
            continue;
        };
        for slot in &component.declaration.slots {
            let expected = match slot.kind {
                SlotKind::Video => ResourceKind::Video,
                SlotKind::Audio => ResourceKind::Audio,
                _ => continue,
            };
            let Some(fill) = instance.fills.get(&slot.name) else {
                continue;
            };
            let Some((resource, span)) = media_reference(&file.path, &file.source, fill)? else {
                continue;
            };
            let Some(actual) = resources.get(resource.as_str()) else {
                continue;
            };
            if *actual != expected {
                return Err(Diagnostic::new(
                    "PROGRAM_SLOT_RESOURCE_KIND",
                    &file.path,
                    format!(
                        "{} slot `{}` requires a {} resource, but `{resource}` is {}",
                        slot_name(slot.kind),
                        slot.name,
                        resource_name(expected),
                        resource_name(*actual)
                    ),
                    span,
                ));
            }
        }
    }
    Ok(())
}

fn media_reference(
    path: &str,
    source: &str,
    fill: &RawBlock,
) -> Result<Option<(String, Span)>, Diagnostic> {
    let raw = &source[fill.content_span.start..fill.content_span.end];
    let tokens = lexer::lex(path, raw).map_err(|errors| errors[0].clone())?;
    let words = tokens.iter().take(4).map(Token::word).collect::<Vec<_>>();
    let [Some("source"), Some("media"), Some("resource"), Some(resource)] = words.as_slice() else {
        return Ok(None);
    };
    let relative = tokens[3].span;
    Ok(Some((
        (*resource).to_owned(),
        Span {
            start: fill.content_span.start + relative.start,
            end: fill.content_span.start + relative.end,
        },
    )))
}

fn slot_name(kind: SlotKind) -> &'static str {
    match kind {
        SlotKind::Video => "video",
        SlotKind::Audio => "audio",
        _ => "media",
    }
}

fn resource_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Video => "video",
        ResourceKind::Audio => "audio",
        ResourceKind::Image => "image",
        ResourceKind::Font => "font",
        ResourceKind::Lut1d => "lut-1d",
        ResourceKind::Lut3d => "lut-3d",
    }
}
