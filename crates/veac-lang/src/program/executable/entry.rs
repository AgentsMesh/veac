use crate::authoring::Span;

use super::super::diagnostic::Diagnostic;
use super::super::model::{FunctionDecl, SurfaceFile};
use crate::program::TypeSyntaxKind;

pub(crate) fn validate(file: &SurfaceFile) -> Result<(), Diagnostic> {
    let mut mains = file.functions.iter().filter(|value| value.name == "main");
    let main = mains.next().ok_or_else(|| {
        error(
            file,
            "PROGRAM_EXECUTABLE_MAIN_MISSING",
            "executable entry requires one root-local `fn main(context: Context) -> Project`",
            Span {
                start: file.source().len(),
                end: file.source().len(),
            },
        )
    })?;
    if let Some(duplicate) = mains.next() {
        return Err(error(
            file,
            "PROGRAM_EXECUTABLE_MAIN_DUPLICATE",
            "executable entry declares `main` more than once",
            duplicate.span,
        ));
    }
    validate_signature(file, main)
}

fn validate_signature(file: &SurfaceFile, main: &FunctionDecl) -> Result<(), Diagnostic> {
    if main.parameters.len() != 1 {
        let span = main.parameters.get(1).map_or(main.span, |value| value.span);
        return Err(error(
            file,
            "PROGRAM_EXECUTABLE_MAIN_ARITY",
            "executable `main` requires exactly one parameter",
            span,
        ));
    }
    let parameter = &main.parameters[0];
    if parameter.name != "context" {
        return Err(error(
            file,
            "PROGRAM_EXECUTABLE_MAIN_PARAMETER_NAME",
            "executable `main` parameter must be named `context`",
            parameter.span,
        ));
    }
    if !named(&parameter.type_syntax, "Context") {
        return Err(error(
            file,
            "PROGRAM_EXECUTABLE_MAIN_PARAMETER_TYPE",
            "executable `main` parameter must have type Context",
            parameter.type_syntax.span(),
        ));
    }
    if !named(&main.return_type_syntax, "Project") {
        return Err(error(
            file,
            "PROGRAM_EXECUTABLE_MAIN_RETURN_TYPE",
            "executable `main` must return Project",
            main.return_type_syntax.span(),
        ));
    }
    Ok(())
}

fn named(syntax: &crate::program::TypeSyntax, expected: &str) -> bool {
    matches!(syntax.kind(), TypeSyntaxKind::Named(value) if value.as_ref() == expected)
}

fn error(file: &SurfaceFile, code: &'static str, message: &'static str, span: Span) -> Diagnostic {
    Diagnostic::new(code, &file.path, message, span)
}
