use crate::source_edit::{ExpressionSite, SourceColorField, SourceNodeRef};

use super::super::super::diagnostic::Diagnostic;
use super::super::super::model::SurfaceFile;
use super::super::syntax::{Block, Entry};
use super::super::SourceIndex;

pub(super) fn index(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: SourceNodeRef,
    body: &Block,
) -> Result<(), Diagnostic> {
    for entry in &body.entries {
        let Some(section @ ("input-space" | "working-space" | "output-space" | "basic")) =
            entry.word(0)
        else {
            continue;
        };
        fields(index, file, &target, section, entry)?;
    }
    Ok(())
}

fn fields(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: &SourceNodeRef,
    section: &str,
    entry: &Entry,
) -> Result<(), Diagnostic> {
    let Some(body) = &entry.block else {
        return Ok(());
    };
    for value in &body.entries {
        let Some(field) = field(section, value.word(0)) else {
            continue;
        };
        super::insert(
            index,
            file,
            target.clone(),
            ExpressionSite::PresetColorField { field },
            value,
            1,
        )?;
    }
    Ok(())
}

fn field(section: &str, name: Option<&str>) -> Option<SourceColorField> {
    use SourceColorField as Field;
    Some(match (section, name?) {
        ("input-space", "primaries") => Field::InputPrimaries,
        ("input-space", "transfer") => Field::InputTransfer,
        ("input-space", "matrix") => Field::InputMatrix,
        ("input-space", "range") => Field::InputRange,
        ("working-space", "primaries") => Field::WorkingPrimaries,
        ("working-space", "transfer") => Field::WorkingTransfer,
        ("working-space", "matrix") => Field::WorkingMatrix,
        ("working-space", "range") => Field::WorkingRange,
        ("output-space", "primaries") => Field::OutputPrimaries,
        ("output-space", "transfer") => Field::OutputTransfer,
        ("output-space", "matrix") => Field::OutputMatrix,
        ("output-space", "range") => Field::OutputRange,
        ("basic", "exposure") => Field::BasicExposure,
        ("basic", "temperature") => Field::BasicTemperature,
        ("basic", "tint") => Field::BasicTint,
        ("basic", "highlights") => Field::BasicHighlights,
        ("basic", "shadows") => Field::BasicShadows,
        ("basic", "fade") => Field::BasicFade,
        _ => return None,
    })
}
