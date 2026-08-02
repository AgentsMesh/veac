use crate::source_edit::{ExpressionSite, SourceNodeRef};

use super::super::diagnostic::Diagnostic;
use super::super::model::SurfaceFile;
use super::item;
use super::syntax::{self, Entry};
use super::SourceIndex;

pub(super) fn index(index: &mut SourceIndex, file: &SurfaceFile) -> Result<(), Diagnostic> {
    let Some(project) = &file.project else {
        return Ok(());
    };
    index.register(
        &file.path,
        SourceNodeRef::project(&file.path, &project.name),
        project.name_span,
    )?;
    let body = syntax::parse(&file.path, &file.source, project.body.content_span)?;
    for entry in &body.entries {
        match entry.word(0) {
            Some("resource") => resource(index, file, entry)?,
            Some("sequence") => sequence(index, file, &project.name, entry)?,
            _ => {}
        }
    }
    Ok(())
}

fn resource(index: &mut SourceIndex, file: &SurfaceFile, entry: &Entry) -> Result<(), Diagnostic> {
    let Some(id) = entry.word(2) else {
        return Ok(());
    };
    let Some(span) = entry.word_span(2) else {
        return Ok(());
    };
    let target = SourceNodeRef::resource(&file.path, id);
    index.register(&file.path, target.clone(), span)?;
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for locator in body
        .entries
        .iter()
        .filter(|value| value.word(0) == Some("locator"))
    {
        let field = match locator.word(1) {
            Some("local") => "path",
            Some("remote") => "uri",
            _ => continue,
        };
        let Some(body) = &locator.block else { continue };
        for value in body
            .entries
            .iter()
            .filter(|value| value.word(0) == Some(field))
        {
            insert(
                index,
                file,
                target.clone(),
                ExpressionSite::ResourceLocator,
                value,
                1,
            )?;
        }
    }
    Ok(())
}

fn sequence(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    project: &str,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(id) = entry.word(1) else {
        return Ok(());
    };
    let Some(span) = entry.word_span(1) else {
        return Ok(());
    };
    index.register(
        &file.path,
        SourceNodeRef::sequence(&file.path, project, id),
        span,
    )?;
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for layer in body
        .entries
        .iter()
        .filter(|value| value.word(0) == Some("layer"))
    {
        self::layer(index, file, project, id, layer)?;
    }
    for apply in body
        .entries
        .iter()
        .filter(|value| value.word(0) == Some("apply"))
    {
        self::apply(index, file, project, id, apply)?;
    }
    Ok(())
}

fn layer(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    project: &str,
    sequence: &str,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(id) = entry.word(2) else {
        return Ok(());
    };
    let Some(span) = entry.word_span(2) else {
        return Ok(());
    };
    index.register(
        &file.path,
        SourceNodeRef::layer(&file.path, project, sequence, id),
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
        item::index(index, file, project, sequence, id, value)?;
    }
    Ok(())
}

fn apply(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    project: &str,
    sequence: &str,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(id) = entry.word(1) else {
        return Ok(());
    };
    let Some(span) = entry.word_span(1) else {
        return Ok(());
    };
    index.register(
        &file.path,
        SourceNodeRef::apply(&file.path, project, sequence, id),
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
            let Some(id) = stage.word(2) else { continue };
            let target =
                SourceNodeRef::stage(&file.path, project, sequence, entry.word(1).unwrap(), id);
            item::parameters(index, file, target, stage, 2)?;
        }
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
