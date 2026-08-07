use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::model::{FileKind, SurfaceFile, TypeDeclKind};
use crate::program::token::TokenKind;
use crate::source_edit::{SourceImportRef, SourceNodeRef, TextRange};
use crate::vocabulary::control_uses::static_program as controls;

use super::{IndexedImport, IndexedTopLevelDeclaration, SourceIndex};

pub(super) fn index(index: &mut SourceIndex, file: &SurfaceFile) -> Result<(), Diagnostic> {
    let tokens = crate::program::lexer::lex(&file.path, &file.source)
        .map_err(|errors| first_diagnostic(errors, file))?;
    index
        .modules
        .insert(file.path.clone(), module_range(file, &tokens)?);
    for value in &file.imports {
        let target = SourceImportRef::new(&file.path, &value.alias);
        let range = range(value.span);
        let indexed = IndexedImport {
            path: value.path.clone(),
            source: file.source[range.start..range.end].to_owned(),
            range,
        };
        if index.imports.insert(target, indexed).is_some() {
            return Err(failure(file, value.span, "import alias is ambiguous"));
        }
    }
    for value in &file.inputs {
        insert(
            index,
            file,
            &tokens,
            SourceNodeRef::input(&file.path, &value.name),
            value.span,
            false,
        )?;
    }
    for value in &file.constants {
        insert(
            index,
            file,
            &tokens,
            SourceNodeRef::constant(&file.path, &value.name),
            value.span,
            value.exported,
        )?;
    }
    for value in &file.functions {
        insert(
            index,
            file,
            &tokens,
            SourceNodeRef::function(&file.path, &value.name),
            value.span,
            value.exported,
        )?;
    }
    for value in &file.implementations {
        let target =
            SourceNodeRef::implementation(&file.path, value.target.to_string(), &value.identity);
        index.register(&file.path, target.clone(), value.span)?;
        insert(index, file, &tokens, target, value.span, false)?;
    }
    for value in &file.types {
        let target = match value.kind {
            TypeDeclKind::Struct(_) => SourceNodeRef::structure(&file.path, &value.name),
            TypeDeclKind::Enum(_) => SourceNodeRef::enumeration(&file.path, &value.name),
        };
        insert(index, file, &tokens, target, value.span, value.exported)?;
    }
    for value in &file.temporal {
        let target = super::temporal::target(&file.path, value);
        insert(index, file, &tokens, target, value.span, false)?;
    }
    Ok(())
}

fn insert(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    tokens: &[crate::program::token::Token],
    target: SourceNodeRef,
    span: Span,
    exported: bool,
) -> Result<(), Diagnostic> {
    let span = declaration_span(tokens, span, exported)
        .ok_or_else(|| failure(file, span, "exported declaration has no export modifier"))?;
    let range = range(span);
    let value = IndexedTopLevelDeclaration {
        source: file.source[range.start..range.end].to_owned(),
        range,
    };
    if index.top_levels.insert(target.clone(), value).is_some() {
        return Err(super::ambiguous(&file.path, &target, span));
    }
    Ok(())
}

fn declaration_span(
    tokens: &[crate::program::token::Token],
    span: Span,
    exported: bool,
) -> Option<Span> {
    if !exported {
        return Some(span);
    }
    let declaration = tokens
        .iter()
        .position(|token| token.span.start == span.start)?;
    let export = tokens.get(declaration.checked_sub(1)?)?;
    export
        .word()
        .is_some_and(|word| controls::EXPORT_MODIFIER.matches(word))
        .then(|| export.span.join(span))
}

fn module_range(
    file: &SurfaceFile,
    tokens: &[crate::program::token::Token],
) -> Result<TextRange, Diagnostic> {
    if file.kind == FileKind::Entry {
        return Ok(TextRange {
            start: 0,
            end: file.source.len(),
        });
    }
    let start = tokens
        .iter()
        .find(|token| token.kind == TokenKind::LeftBrace);
    let end = tokens
        .iter()
        .rev()
        .find(|token| token.kind == TokenKind::RightBrace);
    match (start, end) {
        (Some(start), Some(end)) => Ok(TextRange {
            start: start.span.end,
            end: end.span.start,
        }),
        _ => Err(failure(file, Span::default(), "module body is unavailable")),
    }
}

fn first_diagnostic(mut errors: Vec<Diagnostic>, file: &SurfaceFile) -> Diagnostic {
    if errors.is_empty() {
        failure(file, Span::default(), "source tokens are unavailable")
    } else {
        errors.remove(0)
    }
}

fn range(span: Span) -> TextRange {
    TextRange {
        start: span.start,
        end: span.end,
    }
}

fn failure(file: &SurfaceFile, span: Span, message: &str) -> Diagnostic {
    Diagnostic::new("SOURCE_INDEX_STRUCTURE", &file.path, message, span)
}
