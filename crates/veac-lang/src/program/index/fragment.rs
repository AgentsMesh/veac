use crate::program::diagnostic::Diagnostic;
use crate::program::model::{FileKind, SurfaceFile};

pub(crate) fn validate_top_level_fragment(source: &str) -> Result<(), String> {
    let file =
        crate::program::parser::parse_declaration_fragment("<source-edit-declaration>", source)
            .map_err(first_error)?;
    require_one_declaration(&file)
}

pub(crate) fn validate_import_fragment(source: &str) -> Result<(), String> {
    let file = crate::program::parser::parse_executable("<source-edit-import>", source)
        .map_err(first_error)?;
    if file.kind == FileKind::Entry && file.imports.len() == 1 && declaration_count(&file) == 0 {
        Ok(())
    } else {
        Err("import fragment must contain exactly one import".to_owned())
    }
}

pub(crate) fn validate_build_input_fragment(source: &str) -> Result<(), String> {
    let file = crate::program::parser::parse_executable("<source-edit-input>", source)
        .map_err(first_error)?;
    if file.kind == FileKind::Entry
        && file.imports.is_empty()
        && file.inputs.len() == 1
        && declaration_count(&file) == 1
    {
        Ok(())
    } else {
        Err("input fragment must contain exactly one input declaration".to_owned())
    }
}

pub(crate) fn validate_temporal_fragment(source: &str) -> Result<(), String> {
    let file = crate::program::parser::parse_executable("<source-edit-temporal>", source)
        .map_err(first_error)?;
    if file.kind == FileKind::Entry
        && file.imports.is_empty()
        && file.temporal.len() == 1
        && declaration_count(&file) == 1
    {
        Ok(())
    } else {
        Err("temporal fragment must contain exactly one animate declaration".to_owned())
    }
}

fn require_one_declaration(file: &SurfaceFile) -> Result<(), String> {
    if file.imports.is_empty() && declaration_count(file) == 1 {
        Ok(())
    } else {
        Err(
            "declaration fragment must contain exactly one addressable top-level declaration"
                .to_owned(),
        )
    }
}

fn declaration_count(file: &SurfaceFile) -> usize {
    file.inputs.len()
        + file.constants.len()
        + file.functions.len()
        + file.implementations.len()
        + file.types.len()
        + file.temporal.len()
}

fn first_error(errors: Vec<Diagnostic>) -> String {
    errors
        .first()
        .map(|value| value.message.clone())
        .unwrap_or_else(|| "invalid fragment".to_owned())
}
