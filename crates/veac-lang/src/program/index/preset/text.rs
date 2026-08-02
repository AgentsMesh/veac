use crate::source_edit::{
    ExpressionSite, SourceNodeRef, SourceTextLayoutField, SourceTextStyleField,
};

use super::super::super::diagnostic::Diagnostic;
use super::super::super::model::SurfaceFile;
use super::super::syntax::{Block, Entry};
use super::super::SourceIndex;

pub(super) fn style(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: SourceNodeRef,
    body: &Block,
) -> Result<(), Diagnostic> {
    for entry in &body.entries {
        if let Some((field, skip)) = style_field(entry) {
            super::insert(
                index,
                file,
                target.clone(),
                ExpressionSite::PresetTextStyleField { field },
                entry,
                skip,
            )?;
        }
        nested_style(index, file, &target, entry)?;
    }
    Ok(())
}

pub(super) fn layout(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: SourceNodeRef,
    body: &Block,
) -> Result<(), Diagnostic> {
    for entry in &body.entries {
        if let Some(field) = layout_field(entry.word(0)) {
            insert_layout(index, file, &target, field, entry)?;
        }
        if entry.word(0) == Some("path") {
            if let Some(path) = &entry.block {
                for value in &path.entries {
                    if let Some(field) = path_field(value.word(0)) {
                        insert_layout(index, file, &target, field, value)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn style_field(entry: &Entry) -> Option<(SourceTextStyleField, usize)> {
    use SourceTextStyleField as Field;
    Some(match (entry.word(0)?, entry.word(1)) {
        ("font", Some("family")) => (Field::FontFamily, 2),
        ("font", Some("resource")) => (Field::FontResource, 2),
        ("size", _) => (Field::Size, 1),
        ("weight", _) => (Field::Weight, 1),
        ("font-style", _) => (Field::FontStyle, 1),
        ("fill", _) => (Field::Fill, 1),
        ("tracking", _) => (Field::Tracking, 1),
        ("line-height", _) => (Field::LineHeight, 1),
        _ => return None,
    })
}

fn nested_style(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: &SourceNodeRef,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for value in &body.entries {
        let field = nested_field(entry.word(0), value.word(0));
        if let Some(field) = field {
            super::insert(
                index,
                file,
                target.clone(),
                ExpressionSite::PresetTextStyleField { field },
                value,
                1,
            )?;
        }
        if entry.word(0) == Some("shadow") && value.word(0) == Some("offset") {
            shadow_offset(index, file, target, value)?;
        }
    }
    Ok(())
}

fn nested_field(parent: Option<&str>, name: Option<&str>) -> Option<SourceTextStyleField> {
    use SourceTextStyleField as Field;
    match (parent, name) {
        (Some("background"), Some("color")) => Some(Field::BackgroundColor),
        (Some("background"), Some("padding")) => Some(Field::BackgroundPadding),
        (Some("outline"), Some("color")) => Some(Field::OutlineColor),
        (Some("outline"), Some("width")) => Some(Field::OutlineWidth),
        (Some("shadow"), Some("color")) => Some(Field::ShadowColor),
        (Some("shadow"), Some("opacity")) => Some(Field::ShadowOpacity),
        (Some("shadow"), Some("blur")) => Some(Field::ShadowBlur),
        _ => None,
    }
}

fn shadow_offset(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: &SourceNodeRef,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for value in &body.entries {
        let field = match value.word(0) {
            Some("x") => SourceTextStyleField::ShadowOffsetX,
            Some("y") => SourceTextStyleField::ShadowOffsetY,
            _ => continue,
        };
        super::insert(
            index,
            file,
            target.clone(),
            ExpressionSite::PresetTextStyleField { field },
            value,
            1,
        )?;
    }
    Ok(())
}

fn layout_field(name: Option<&str>) -> Option<SourceTextLayoutField> {
    use SourceTextLayoutField as Field;
    match name? {
        "box-width" => Some(Field::BoxWidth),
        "box-height" => Some(Field::BoxHeight),
        "wrap" => Some(Field::Wrap),
        "overflow" => Some(Field::Overflow),
        "horizontal-align" => Some(Field::HorizontalAlign),
        "vertical-align" => Some(Field::VerticalAlign),
        "writing-mode" => Some(Field::WritingMode),
        "orientation" => Some(Field::Orientation),
        _ => None,
    }
}

fn path_field(name: Option<&str>) -> Option<SourceTextLayoutField> {
    use SourceTextLayoutField as Field;
    match name? {
        "start-offset" => Some(Field::PathStartOffset),
        "reverse" => Some(Field::PathReverse),
        "align" => Some(Field::PathAlign),
        _ => None,
    }
}

fn insert_layout(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: &SourceNodeRef,
    field: SourceTextLayoutField,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    super::insert(
        index,
        file,
        target.clone(),
        ExpressionSite::PresetTextLayoutField { field },
        entry,
        1,
    )
}
