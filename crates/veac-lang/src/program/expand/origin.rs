use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::index::syntax::{self, Block, Entry};
use crate::program::token::TokenKind;

pub(super) fn local_declarations(
    path: &str,
    source: &str,
    span: Span,
) -> Result<Vec<(String, Span)>, Diagnostic> {
    let block = syntax::parse(path, source, span)?;
    let mut declarations = Vec::new();
    visit(&block, None, &mut declarations);
    Ok(declarations)
}

fn visit(block: &Block, parent: Option<&str>, declarations: &mut Vec<(String, Span)>) {
    for entry in &block.entries {
        if let Some(index) = declaration_index(parent, entry) {
            if let Some(token) = entry.head.get(index) {
                if let TokenKind::LocalId(value) = &token.kind {
                    declarations.push((value.clone(), token.span));
                }
            }
        }
        if let Some(block) = &entry.block {
            visit(block, entry.word(0), declarations);
        }
    }
}

fn declaration_index(parent: Option<&str>, entry: &Entry) -> Option<usize> {
    match (parent, entry.word(0)) {
        (None, Some("layer" | "relation")) => Some(2),
        (None, Some("apply")) => Some(1),
        (Some("layer"), Some("item")) => Some(1),
        (Some("mapping"), Some("key")) => Some(1),
        (Some("pipeline"), Some("stage")) => Some(2),
        (Some("mix"), Some("mask")) => Some(1),
        (Some("modifiers"), _) => Some(1),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
