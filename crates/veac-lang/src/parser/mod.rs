mod timeline;

use std::path::PathBuf;

use crate::ast::*;
use crate::error::{ErrorKind, VeacError};
use crate::token::{Token, TokenKind};

/// Recursive descent parser for VEAC.
pub struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse the token stream into a complete program AST.
    pub fn parse(&mut self) -> Result<Program, VeacError> {
        let mut program = Program {
            includes: Vec::new(),
            project: None,
            assets: Vec::new(),
            variables: Vec::new(),
            timelines: Vec::new(),
            outputs: Vec::new(),
        };

        while !self.is_at_end() {
            match &self.current().kind {
                TokenKind::Include => program.includes.push(self.parse_include()?),
                TokenKind::Project => {
                    if program.project.is_some() {
                        return Err(VeacError::new(
                            ErrorKind::DuplicateDefinition,
                            "duplicate `project` declaration",
                            Some(self.current().span),
                        )
                        .with_hint("only one `project` block is allowed per file"));
                    }
                    program.project = Some(self.parse_project()?);
                }
                TokenKind::Asset => program.assets.push(self.parse_asset()?),
                TokenKind::Let => program.variables.push(self.parse_let()?),
                TokenKind::Timeline => program.timelines.push(self.parse_timeline()?),
                TokenKind::Ident(s) if s == "output" => program.outputs.push(self.parse_output()?),
                TokenKind::Eof => break,
                _ => {
                    return Err(VeacError::new(
                        ErrorKind::UnexpectedToken,
                        format!("unexpected {} at top level", self.current().kind),
                        Some(self.current().span),
                    )
                    .with_hint(
                        "expected `project`, `asset`, `let`, `timeline`, `output`, or `include`",
                    ));
                }
            }
        }

        Ok(program)
    }

    // --- Helpers ---

    pub(crate) fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    pub(crate) fn is_at_end(&self) -> bool {
        self.current().kind == TokenKind::Eof
    }

    pub(crate) fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if !self.is_at_end() {
            self.pos += 1;
        }
        tok
    }

    pub(crate) fn expect(&mut self, expected: &TokenKind) -> Result<&Token, VeacError> {
        if std::mem::discriminant(&self.current().kind) == std::mem::discriminant(expected) {
            Ok(self.advance())
        } else {
            Err(VeacError::new(
                ErrorKind::ExpectedToken,
                format!("expected {expected}, found {}", self.current().kind),
                Some(self.current().span),
            ))
        }
    }

    pub(crate) fn expect_string(&mut self) -> Result<String, VeacError> {
        match &self.current().kind {
            TokenKind::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            _ => Err(VeacError::new(
                ErrorKind::ExpectedToken,
                format!("expected string, found {}", self.current().kind),
                Some(self.current().span),
            )),
        }
    }

    pub(crate) fn expect_ident(&mut self) -> Result<String, VeacError> {
        match &self.current().kind {
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(VeacError::new(
                ErrorKind::ExpectedToken,
                format!("expected identifier, found {}", self.current().kind),
                Some(self.current().span),
            )),
        }
    }

    // --- Top-level declarations ---

    fn parse_project(&mut self) -> Result<ProjectDecl, VeacError> {
        self.expect(&TokenKind::Project)?;
        let name = self.expect_string()?;
        self.expect(&TokenKind::LBrace)?;
        let attributes = self.parse_attributes()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(ProjectDecl { name, attributes })
    }

    fn parse_asset(&mut self) -> Result<AssetDecl, VeacError> {
        self.expect(&TokenKind::Asset)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Equals)?;

        let kind = match &self.current().kind {
            TokenKind::Video => AssetKind::Video,
            TokenKind::Audio => AssetKind::Audio,
            TokenKind::Image => AssetKind::Image,
            _ => {
                return Err(VeacError::new(
                    ErrorKind::ExpectedToken,
                    format!(
                        "expected `video`, `audio`, or `image`, found {}",
                        self.current().kind
                    ),
                    Some(self.current().span),
                ));
            }
        };
        self.advance();

        self.expect(&TokenKind::LParen)?;
        let path_str = self.expect_string()?;
        self.expect(&TokenKind::RParen)?;

        Ok(AssetDecl {
            name,
            kind,
            path: PathBuf::from(path_str),
        })
    }

    fn parse_let(&mut self) -> Result<LetDecl, VeacError> {
        self.expect(&TokenKind::Let)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Equals)?;
        let value = self.parse_expression()?;
        Ok(LetDecl { name, value })
    }

    fn parse_include(&mut self) -> Result<IncludeDecl, VeacError> {
        self.expect(&TokenKind::Include)?;
        let path_str = self.expect_string()?;
        Ok(IncludeDecl {
            path: PathBuf::from(path_str),
        })
    }

    fn parse_output(&mut self) -> Result<OutputDecl, VeacError> {
        // "output" is an ident, not a keyword
        self.advance(); // skip "output"
        let path_str = self.expect_string()?;
        self.expect(&TokenKind::LBrace)?;
        let attributes = self.parse_attributes()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(OutputDecl {
            path: PathBuf::from(path_str),
            attributes,
        })
    }
}
