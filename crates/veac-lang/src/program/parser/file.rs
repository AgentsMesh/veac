use super::Parser;
use crate::program::diagnostic::Diagnostic;
use crate::program::lexer;
use crate::program::model::{FileKind, SurfaceFile};
use crate::program::token::TokenKind;
use crate::vocabulary::control_uses::static_program as controls;

mod dispatch;

pub(crate) fn parse(path: &str, source: &str) -> Result<SurfaceFile, Vec<Diagnostic>> {
    let document = lexer::lex_document(path, source)?;
    let mut parser = Parser::new(path, document);
    parse_file(&mut parser).map_err(|error| vec![error])
}

pub(crate) fn parse_executable(path: &str, source: &str) -> Result<SurfaceFile, Vec<Diagnostic>> {
    parse(path, source)
}

pub(crate) fn parse_declaration_fragment(
    path: &str,
    source: &str,
) -> Result<SurfaceFile, Vec<Diagnostic>> {
    let document = lexer::lex_document(path, source)?;
    let mut parser = Parser::new(path, document);
    let mut file = empty(&parser, FileKind::Entry);
    while !parser.at_eof() {
        let exported = if parser.at_control(controls::EXPORT_MODIFIER) {
            parser.advance();
            true
        } else {
            false
        };
        dispatch::parse(&mut parser, &mut file, exported).map_err(|error| vec![error])?;
    }
    Ok(file)
}

fn parse_file(parser: &mut Parser<'_>) -> Result<SurfaceFile, Diagnostic> {
    if parser.at_control(controls::MODULE_DECLARATION) {
        module(parser)
    } else {
        entry(parser)
    }
}

fn empty(parser: &Parser<'_>, kind: FileKind) -> SurfaceFile {
    SurfaceFile {
        path: parser.path.to_owned(),
        syntax: parser.document.clone(),
        kind,
        imports: Vec::new(),
        inputs: Vec::new(),
        constants: Vec::new(),
        functions: Vec::new(),
        implementations: Vec::new(),
        types: Vec::new(),
        temporal: Vec::new(),
    }
}

fn entry(parser: &mut Parser<'_>) -> Result<SurfaceFile, Diagnostic> {
    let mut file = empty(parser, FileKind::Entry);
    while !parser.at_eof() {
        dispatch::parse(parser, &mut file, false)?;
    }
    Ok(file)
}

fn module(parser: &mut Parser<'_>) -> Result<SurfaceFile, Diagnostic> {
    parser.expect_control(controls::MODULE_DECLARATION)?;
    parser.expect(TokenKind::LeftBrace, "`{`")?;
    let mut file = empty(parser, FileKind::Module);
    while !parser.at(&TokenKind::RightBrace) && !parser.at_eof() {
        if parser.at_control(controls::INPUT_DECLARATION) {
            return Err(parser.error(
                "PROGRAM_INPUT_ENTRY_ONLY",
                "input declarations belong to the executable entry source",
                parser.current().span,
            ));
        }
        if parser.at_control(controls::TEMPORAL_DECLARATION) {
            return Err(parser.error(
                "PROGRAM_TEMPORAL_ENTRY_ONLY",
                "animate declarations belong to the executable entry source",
                parser.current().span,
            ));
        }
        let exported = if parser.at_control(controls::EXPORT_MODIFIER) {
            parser.advance();
            true
        } else {
            false
        };
        dispatch::parse(parser, &mut file, exported)?;
    }
    parser.expect(TokenKind::RightBrace, "`}`")?;
    if !parser.at_eof() {
        return Err(parser.error(
            "PROGRAM_TRAILING_DECLARATION",
            "module closing brace must end the file",
            parser.current().span,
        ));
    }
    Ok(file)
}

#[cfg(test)]
#[path = "file/tests.rs"]
mod tests;
