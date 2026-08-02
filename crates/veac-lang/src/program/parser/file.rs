use super::{component, declaration, instance, Parser};
use crate::program::diagnostic::Diagnostic;
use crate::program::lexer;
use crate::program::model::{FileKind, ProjectDecl, SurfaceFile};
use crate::program::token::TokenKind;

pub(crate) fn parse(path: &str, source: &str) -> Result<SurfaceFile, Vec<Diagnostic>> {
    let tokens = lexer::lex(path, source)?;
    let mut parser = Parser::new(path, source, tokens);
    parse_file(&mut parser).map_err(|error| vec![error])
}

fn parse_file(parser: &mut Parser<'_>) -> Result<SurfaceFile, Diagnostic> {
    if parser.at_word("module") {
        module(parser)
    } else {
        entry(parser)
    }
}

fn empty(path: &str, source: &str, kind: FileKind) -> SurfaceFile {
    SurfaceFile {
        path: path.to_owned(),
        source: source.to_owned(),
        kind,
        imports: Vec::new(),
        constants: Vec::new(),
        presets: Vec::new(),
        components: Vec::new(),
        instances: Vec::new(),
        project: None,
    }
}

fn entry(parser: &mut Parser<'_>) -> Result<SurfaceFile, Diagnostic> {
    let mut file = empty(parser.path, parser.source, FileKind::Entry);
    while !parser.at_eof() {
        if parser.at_word("project") {
            if file.project.is_some() {
                return Err(parser.error(
                    "PROGRAM_DUPLICATE_PROJECT",
                    "entry contains more than one project",
                    parser.current().span,
                ));
            }
            file.project = Some(project(parser)?);
        } else {
            declaration(parser, &mut file, false)?;
        }
    }
    if file.project.is_none() {
        return Err(parser.error(
            "PROGRAM_PROJECT_REQUIRED",
            "entry file requires one project",
            parser.current().span,
        ));
    }
    Ok(file)
}

fn module(parser: &mut Parser<'_>) -> Result<SurfaceFile, Diagnostic> {
    parser.expect_word("module")?;
    parser.expect(TokenKind::LeftBrace, "`{`")?;
    let mut file = empty(parser.path, parser.source, FileKind::Module);
    while !parser.at(&TokenKind::RightBrace) && !parser.at_eof() {
        if parser.at_word("instance") {
            return Err(parser.error(
                "PROGRAM_MODULE_INSTANCE",
                "component instances belong in an entry source",
                parser.current().span,
            ));
        }
        let exported = if parser.at_word("export") {
            parser.advance();
            true
        } else {
            false
        };
        declaration(parser, &mut file, exported)?;
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

fn declaration(
    parser: &mut Parser<'_>,
    file: &mut SurfaceFile,
    exported: bool,
) -> Result<(), Diagnostic> {
    if parser.at_word("import") && !exported {
        file.imports.push(declaration::import(parser)?);
    } else if parser.at_word("const") {
        file.constants
            .push(declaration::constant(parser, exported)?);
    } else if parser.at_word("preset") {
        file.presets.push(declaration::preset(parser, exported)?);
    } else if parser.at_word("component") {
        file.components.push(component::parse(parser, exported)?);
    } else if parser.at_word("instance") && !exported {
        let value = instance::top_level(parser)?;
        if file
            .instances
            .iter()
            .any(|candidate| candidate.id == value.id)
        {
            return Err(parser.error(
                "PROGRAM_DUPLICATE_INSTANCE",
                format!(
                    "component instance `{}` is declared more than once",
                    value.id
                ),
                value.span,
            ));
        }
        file.instances.push(value);
    } else {
        return Err(parser.error(
            "PROGRAM_DECLARATION",
            "expected import, const, preset, component, instance, or project",
            parser.current().span,
        ));
    }
    Ok(())
}

fn project(parser: &mut Parser<'_>) -> Result<ProjectDecl, Diagnostic> {
    let start = parser.expect_word("project")?;
    let (name, name_span) = parser.identifier("project ID")?;
    let body = parser.raw_block()?;
    let span = start.join(body.span);
    Ok(ProjectDecl {
        name,
        name_span,
        body,
        span,
    })
}
