use crate::source_edit::{ExpressionSite, SourceNodeRef};

use super::super::diagnostic::Diagnostic;
use super::super::model::SurfaceFile;
use super::syntax::Entry;
use super::SourceIndex;

pub(super) fn index(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    project: &str,
    sequence: &str,
    layer: &str,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(id) = entry.word(1) else {
        return Ok(());
    };
    let Some(span) = entry.word_span(1) else {
        return Ok(());
    };
    let target = SourceNodeRef::item(&file.path, project, sequence, layer, id);
    index.register(&file.path, target.clone(), span)?;
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for member in &body.entries {
        match member.word(0) {
            Some("record") => record(index, file, &target, member)?,
            Some("state") => enabled(index, file, &target, member)?,
            Some("source") => text(index, file, &target, member)?,
            Some("modifiers") => modifiers(index, file, project, sequence, layer, id, member)?,
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn enabled(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: &SourceNodeRef,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for playback in body
        .entries
        .iter()
        .filter(|value| value.word(0) == Some("playback"))
    {
        insert(
            index,
            file,
            target.clone(),
            ExpressionSite::ItemEnabled,
            playback,
            1,
        )?;
    }
    Ok(())
}

pub(super) fn record(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: &SourceNodeRef,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for field in &body.entries {
        let site = match field.word(0) {
            Some("at") => ExpressionSite::ItemRecordStart,
            Some("duration") => ExpressionSite::ItemRecordDuration,
            _ => continue,
        };
        insert(index, file, target.clone(), site, field, 1)?;
    }
    Ok(())
}

pub(super) fn text(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: &SourceNodeRef,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    if !matches!(entry.word(1), Some("text" | "caption")) {
        return Ok(());
    }
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for content in body
        .entries
        .iter()
        .filter(|value| value.word(0) == Some("content"))
    {
        insert(
            index,
            file,
            target.clone(),
            ExpressionSite::TextContent,
            content,
            1,
        )?;
    }
    Ok(())
}

fn modifiers(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    project: &str,
    sequence: &str,
    layer: &str,
    item: &str,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for entry in &body.entries {
        let Some(id) = entry.word(1) else { continue };
        let target = SourceNodeRef::modifier(&file.path, project, sequence, layer, item, id);
        parameters(index, file, target, entry, 1)?;
    }
    Ok(())
}

pub(super) fn parameters(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: SourceNodeRef,
    entry: &Entry,
    id_index: usize,
) -> Result<(), Diagnostic> {
    let Some(span) = entry.word_span(id_index) else {
        return Ok(());
    };
    index.register(&file.path, target.clone(), span)?;
    let Some(parameters) = &entry.block else {
        return Ok(());
    };
    for parameter in parameters
        .entries
        .iter()
        .filter(|value| value.word(0) == Some("parameter"))
    {
        let Some(name) = parameter.word(1) else {
            continue;
        };
        insert(
            index,
            file,
            target.clone(),
            ExpressionSite::ModifierParameter {
                parameter: name.to_owned(),
            },
            parameter,
            2,
        )?;
    }
    Ok(())
}

fn insert(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: SourceNodeRef,
    site: ExpressionSite,
    entry: &Entry,
    skip: usize,
) -> Result<(), Diagnostic> {
    let Some((source, span)) = entry.expression(&file.source, skip) else {
        return Ok(());
    };
    index.insert(&file.path, target, site, source, span)
}
