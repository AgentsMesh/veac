use crate::source_edit::{ExpressionSite, SourceNodeRef};

use super::super::diagnostic::Diagnostic;
use super::super::model::{ComponentDecl, SurfaceFile};
use super::item;
use super::syntax::{self, Entry};
use super::SourceIndex;

pub(super) fn index(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    component: &ComponentDecl,
) -> Result<(), Diagnostic> {
    local_instances(index, file, component)?;
    let body = syntax::parse(&file.path, &file.source, component.body.content_span)?;
    for entry in &body.entries {
        match entry.word(0) {
            Some("layer") => layer(index, file, &component.name, entry)?,
            Some("apply") => apply(index, file, &component.name, entry)?,
            _ => {}
        }
    }
    Ok(())
}

fn local_instances(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    component: &ComponentDecl,
) -> Result<(), Diagnostic> {
    let root = syntax::parse(&file.path, &file.source, component.span)?;
    let Some(members) = root.entries.first().and_then(|value| value.block.as_ref()) else {
        return Ok(());
    };
    for entry in members
        .entries
        .iter()
        .filter(|value| value.word(0) == Some("instance"))
    {
        let Some(id) = entry.id(2) else { continue };
        let Some(span) = entry.word_span(2) else {
            continue;
        };
        let Some(instance) = component.instances.iter().find(|value| value.id == id) else {
            continue;
        };
        let target = SourceNodeRef::component_local_instance(&file.path, &component.name, id);
        index.register(&file.path, target.clone(), span)?;
        for (parameter, binding) in &instance.bindings {
            index.insert(
                &file.path,
                target.clone(),
                ExpressionSite::ComponentLocalInstanceArgument {
                    parameter: parameter.clone(),
                },
                &binding.source,
                binding.span,
            )?;
        }
    }
    Ok(())
}

fn layer(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    component: &str,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(id) = entry.id(2) else { return Ok(()) };
    let Some(span) = entry.word_span(2) else {
        return Ok(());
    };
    index.register(
        &file.path,
        SourceNodeRef::component_layer(&file.path, component, id),
        span,
    )?;
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for value in body
        .entries
        .iter()
        .filter(|value| value.word(0) == Some("item"))
    {
        self::item(index, file, component, id, value)?;
    }
    Ok(())
}

fn item(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    component: &str,
    layer: &str,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(id) = entry.id(1) else { return Ok(()) };
    let Some(span) = entry.word_span(1) else {
        return Ok(());
    };
    let target = SourceNodeRef::component_item(&file.path, component, layer, id);
    index.register(&file.path, target.clone(), span)?;
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for member in &body.entries {
        match member.word(0) {
            Some("record") => item::record(index, file, &target, member)?,
            Some("state") => item::enabled(index, file, &target, member)?,
            Some("source") => item::text(index, file, &target, member)?,
            Some("modifiers") => modifiers(index, file, component, layer, id, member)?,
            _ => {}
        }
    }
    Ok(())
}

fn modifiers(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    component: &str,
    layer: &str,
    item: &str,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for modifier in &body.entries {
        let Some(id) = modifier.id(1) else { continue };
        let target = SourceNodeRef::component_modifier(&file.path, component, layer, item, id);
        item::parameters(index, file, target, modifier, 1)?;
    }
    Ok(())
}

fn apply(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    component: &str,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(id) = entry.id(1) else { return Ok(()) };
    let Some(span) = entry.word_span(1) else {
        return Ok(());
    };
    index.register(
        &file.path,
        SourceNodeRef::component_apply(&file.path, component, id),
        span,
    )?;
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for pipeline in body
        .entries
        .iter()
        .filter(|value| value.word(0) == Some("pipeline"))
    {
        let Some(stages) = &pipeline.block else {
            continue;
        };
        for stage in stages
            .entries
            .iter()
            .filter(|value| value.word(0) == Some("stage"))
        {
            let Some(stage_id) = stage.id(2) else {
                continue;
            };
            let target = SourceNodeRef::component_stage(&file.path, component, id, stage_id);
            item::parameters(index, file, target, stage, 2)?;
        }
    }
    Ok(())
}
