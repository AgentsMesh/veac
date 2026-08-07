use super::super::{declaration, function, input, method, temporal, type_declaration, Parser};
use crate::program::diagnostic::Diagnostic;
use crate::program::model::SurfaceFile;
use crate::vocabulary::control_uses::static_program as controls;

pub(super) fn parse(
    parser: &mut Parser<'_>,
    file: &mut SurfaceFile,
    exported: bool,
) -> Result<(), Diagnostic> {
    if parser.at_control(controls::IMPORT_DECLARATION) && !exported {
        file.imports.push(declaration::import(parser)?);
    } else if parser.at_control(controls::INPUT_DECLARATION) && !exported {
        file.inputs.push(input::parse(parser)?);
    } else if parser.at_control(controls::INPUT_DECLARATION) {
        return Err(parser.error(
            "PROGRAM_INPUT_EXPORT",
            "input declarations are root-local and cannot be exported",
            parser.current().span,
        ));
    } else if parser.at_control(controls::CONST_DECLARATION) {
        file.constants
            .push(declaration::constant(parser, exported)?);
    } else if parser.at_control(controls::FUNCTION_DECLARATION) {
        file.functions.push(function::parse(parser, exported)?);
    } else if parser.at_control(controls::IMPL_DECLARATION) && exported {
        return Err(parser.error(
            "PROGRAM_IMPL_EXPORT",
            "implementation blocks are not exported; export individual methods",
            parser.current().span,
        ));
    } else if parser.at_control(controls::IMPL_DECLARATION) {
        file.implementations.push(method::parse(parser)?);
    } else if parser.at_control(controls::STRUCT_DECLARATION) {
        check_type_limit(parser, file)?;
        file.types
            .push(type_declaration::structure(parser, exported)?);
    } else if parser.at_control(controls::ENUM_DECLARATION) {
        check_type_limit(parser, file)?;
        file.types
            .push(type_declaration::enumeration(parser, exported)?);
    } else if parser.at_control(controls::TEMPORAL_DECLARATION) && !exported {
        file.temporal.push(temporal::parse(parser)?);
    } else {
        return Err(parser.error(
            "PROGRAM_DECLARATION",
            "expected import, input, const, fn, impl, struct, enum, or animate",
            parser.current().span,
        ));
    }
    Ok(())
}

fn check_type_limit(parser: &Parser<'_>, file: &SurfaceFile) -> Result<(), Diagnostic> {
    if file.types.len() < crate::program::MAX_TYPE_DECLARATIONS {
        Ok(())
    } else {
        Err(parser.error(
            "PROGRAM_TYPE_DECLARATION_LIMIT",
            format!(
                "source exceeds the {} nominal type declaration limit",
                crate::program::MAX_TYPE_DECLARATIONS
            ),
            parser.current().span,
        ))
    }
}
