use crate::authoring::Span;

use super::super::diagnostic::Diagnostic;
use super::super::lexer;
use super::super::token::{Token, TokenKind};

#[derive(Debug)]
pub(crate) struct Block {
    pub(crate) entries: Vec<Entry>,
}

#[derive(Debug)]
pub(crate) struct Entry {
    pub(crate) head: Vec<Token>,
    pub(crate) block: Option<Block>,
}

impl Entry {
    pub fn word(&self, index: usize) -> Option<&str> {
        self.head.get(index).and_then(Token::word)
    }

    pub fn id(&self, index: usize) -> Option<&str> {
        match &self.head.get(index)?.kind {
            TokenKind::Word(value) | TokenKind::LocalId(value) => Some(value),
            _ => None,
        }
    }

    pub fn word_span(&self, index: usize) -> Option<Span> {
        self.head.get(index).map(|token| token.span)
    }

    pub fn expression<'a>(&self, source: &'a str, skip: usize) -> Option<(&'a str, Span)> {
        if self.block.is_some() {
            return None;
        }
        let values = self.head.get(skip..)?;
        let span = values.first()?.span.join(values.last()?.span);
        Some((&source[span.start..span.end], span))
    }
}

pub(crate) fn parse(path: &str, source: &str, span: Span) -> Result<Block, Diagnostic> {
    let tokens = lexer::lex(path, source).map_err(|errors| errors[0].clone())?;
    let tokens = tokens
        .into_iter()
        .filter(|token| {
            !matches!(token.kind, TokenKind::Eof)
                && token.span.start >= span.start
                && token.span.end <= span.end
        })
        .collect();
    Parser {
        path,
        tokens,
        cursor: 0,
    }
    .block(false)
}

pub(crate) fn validate_expression_fragment(source: &str) -> Result<(), String> {
    let wrapped = format!("source_edit_value {source};");
    let block = parse(
        "<source-edit-expression>",
        &wrapped,
        Span {
            start: 0,
            end: wrapped.len(),
        },
    )
    .map_err(|error| error.message)?;
    let [entry] = block.entries.as_slice() else {
        return Err("replacement must contain exactly one expression".into());
    };
    if entry.word(0) != Some("source_edit_value") || entry.block.is_some() || entry.head.len() < 2 {
        return Err("replacement must be one leaf expression without a block".into());
    }
    Ok(())
}

struct Parser<'a> {
    path: &'a str,
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser<'_> {
    fn block(&mut self, nested: bool) -> Result<Block, Diagnostic> {
        let mut entries = Vec::new();
        while self.cursor < self.tokens.len() {
            if matches!(self.current().kind, TokenKind::RightBrace) {
                if nested {
                    self.cursor += 1;
                    return Ok(Block { entries });
                }
                return Err(self.error("unexpected closing brace", self.current().span));
            }
            entries.push(self.entry()?);
        }
        if nested {
            Err(self.error("nested source block is not closed", Span::default()))
        } else {
            Ok(Block { entries })
        }
    }

    fn entry(&mut self) -> Result<Entry, Diagnostic> {
        let mut head = Vec::new();
        while self.cursor < self.tokens.len() {
            match self.current().kind {
                TokenKind::Semicolon => {
                    self.cursor += 1;
                    return self.finish(head, None);
                }
                TokenKind::LeftBrace => {
                    self.cursor += 1;
                    let block = self.block(true)?;
                    return self.finish(head, Some(block));
                }
                TokenKind::DollarLeftBrace => self.placeholder(&mut head)?,
                TokenKind::RightBrace => {
                    return Err(
                        self.error("source entry requires `;` or a block", self.current().span)
                    );
                }
                _ => {
                    head.push(self.current().clone());
                    self.cursor += 1;
                }
            }
        }
        Err(self.error("source entry is not terminated", Span::default()))
    }

    fn placeholder(&mut self, head: &mut Vec<Token>) -> Result<(), Diagnostic> {
        let start = self.current().span;
        let mut depth = 0usize;
        while self.cursor < self.tokens.len() {
            let token = self.current().clone();
            self.cursor += 1;
            match token.kind {
                TokenKind::DollarLeftBrace | TokenKind::LeftBrace => depth += 1,
                TokenKind::RightBrace => depth = depth.saturating_sub(1),
                _ => {}
            }
            head.push(token);
            if depth == 0 {
                return Ok(());
            }
        }
        Err(self.error("expression placeholder is not closed", start))
    }

    fn finish(&self, head: Vec<Token>, block: Option<Block>) -> Result<Entry, Diagnostic> {
        if head.is_empty() {
            Err(self.error("source entry has no name", Span::default()))
        } else {
            Ok(Entry { head, block })
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.cursor]
    }

    fn error(&self, message: &str, span: Span) -> Diagnostic {
        Diagnostic::new("SOURCE_INDEX_SYNTAX", self.path, message, span)
    }
}
