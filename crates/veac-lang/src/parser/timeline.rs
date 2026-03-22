use std::path::PathBuf;

use crate::ast::*;
use crate::error::{ErrorKind, VeacError};
use crate::token::TokenKind;

use super::Parser;

/// Timeline, track, clip, text, attribute and expression parsing.
impl Parser {
    pub(crate) fn parse_timeline(&mut self) -> Result<TimelineDecl, VeacError> {
        self.expect(&TokenKind::Timeline)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;

        let mut tracks = Vec::new();
        while self.current().kind != TokenKind::RBrace {
            tracks.push(self.parse_track()?);
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(TimelineDecl { name, tracks })
    }

    fn parse_track(&mut self) -> Result<TrackDecl, VeacError> {
        self.expect(&TokenKind::Track)?;

        let kind = match &self.current().kind {
            TokenKind::Video => TrackKind::Video,
            TokenKind::Audio => TrackKind::Audio,
            TokenKind::Text => TrackKind::Text,
            TokenKind::Ident(s) if s == "overlay" => TrackKind::Overlay,
            _ => {
                return Err(VeacError::new(
                    ErrorKind::ExpectedToken,
                    format!(
                        "expected `video`, `audio`, `text`, or `overlay` after `track`, found {}",
                        self.current().kind
                    ),
                    Some(self.current().span),
                )
                .with_hint("track type must be `video`, `audio`, `text`, or `overlay`"));
            }
        };
        self.advance();
        self.expect(&TokenKind::LBrace)?;

        let mut items = Vec::new();
        while self.current().kind != TokenKind::RBrace {
            match &self.current().kind {
                TokenKind::Clip => items.push(TrackItem::Clip(self.parse_clip()?)),
                TokenKind::Text => items.push(TrackItem::TextOverlay(self.parse_text_overlay()?)),
                TokenKind::Transition => {
                    items.push(TrackItem::Transition(self.parse_transition()?))
                }
                TokenKind::Image => {
                    items.push(TrackItem::ImageOverlay(self.parse_image_overlay()?))
                }
                TokenKind::Ident(s) if s == "gap" => items.push(TrackItem::Gap(self.parse_gap()?)),
                TokenKind::Ident(s) if s == "freeze" => {
                    items.push(TrackItem::Freeze(self.parse_freeze()?))
                }
                TokenKind::Ident(s) if s == "pip" => items.push(TrackItem::Pip(self.parse_pip()?)),
                TokenKind::Ident(s) if s == "subtitle" => {
                    items.push(TrackItem::Subtitle(self.parse_subtitle()?))
                }
                _ => {
                    return Err(VeacError::new(
                        ErrorKind::UnexpectedToken,
                        format!("unexpected {} in track", self.current().kind),
                        Some(self.current().span),
                    ));
                }
            }
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(TrackDecl { kind, items })
    }

    fn parse_clip(&mut self) -> Result<ClipDecl, VeacError> {
        self.expect(&TokenKind::Clip)?;
        let asset_ref = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let attributes = self.parse_attributes()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(ClipDecl {
            asset_ref,
            attributes,
        })
    }

    fn parse_text_overlay(&mut self) -> Result<TextOverlayDecl, VeacError> {
        self.expect(&TokenKind::Text)?;
        let content = self.expect_string()?;
        self.expect(&TokenKind::LBrace)?;
        let attributes = self.parse_attributes()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(TextOverlayDecl {
            content,
            attributes,
        })
    }

    fn parse_transition(&mut self) -> Result<TransitionDecl, VeacError> {
        self.expect(&TokenKind::Transition)?;
        self.expect(&TokenKind::LBrace)?;
        let attributes = self.parse_attributes()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(TransitionDecl { attributes })
    }

    fn parse_image_overlay(&mut self) -> Result<ImageOverlayDecl, VeacError> {
        self.expect(&TokenKind::Image)?;
        let asset_ref = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let attributes = self.parse_attributes()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(ImageOverlayDecl {
            asset_ref,
            attributes,
        })
    }

    fn parse_gap(&mut self) -> Result<GapDecl, VeacError> {
        self.advance(); // skip "gap" ident
        self.expect(&TokenKind::LBrace)?;
        let attributes = self.parse_attributes()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(GapDecl { attributes })
    }

    fn parse_freeze(&mut self) -> Result<FreezeDecl, VeacError> {
        self.advance(); // skip "freeze" ident
        let asset_ref = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let attributes = self.parse_attributes()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(FreezeDecl {
            asset_ref,
            attributes,
        })
    }

    fn parse_pip(&mut self) -> Result<PipDecl, VeacError> {
        self.advance(); // skip "pip" ident
        let asset_ref = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let attributes = self.parse_attributes()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(PipDecl {
            asset_ref,
            attributes,
        })
    }

    fn parse_subtitle(&mut self) -> Result<SubtitleDecl, VeacError> {
        self.advance(); // skip "subtitle" ident
        let path_str = self.expect_string()?;
        self.expect(&TokenKind::LBrace)?;
        let attributes = self.parse_attributes()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(SubtitleDecl {
            path: PathBuf::from(path_str),
            attributes,
        })
    }

    pub(crate) fn parse_attributes(&mut self) -> Result<Vec<Attribute>, VeacError> {
        let mut attrs = Vec::new();

        while self.current().kind != TokenKind::RBrace {
            match &self.current().kind {
                TokenKind::Track
                | TokenKind::Clip
                | TokenKind::Text
                | TokenKind::Transition
                | TokenKind::Image => break,
                // Also break on known ident-based track item keywords
                TokenKind::Ident(s)
                    if matches!(s.as_str(), "gap" | "freeze" | "pip" | "subtitle") =>
                {
                    break
                }
                _ => {}
            }

            let key = self.expect_ident()?;
            self.expect(&TokenKind::Equals)?;
            let value = self.parse_expression()?;
            attrs.push(Attribute { key, value });
        }

        Ok(attrs)
    }

    pub(crate) fn parse_expression(&mut self) -> Result<Expression, VeacError> {
        let tok = self.current().clone();
        match &tok.kind {
            TokenKind::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::StringLit(s))
            }
            TokenKind::IntLit(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::IntLit(n))
            }
            TokenKind::FloatLit(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::FloatLit(n))
            }
            TokenKind::BoolLit(b) => {
                let b = *b;
                self.advance();
                Ok(Expression::BoolLit(b))
            }
            TokenKind::TimeSec(v) => {
                let v = *v;
                self.advance();
                Ok(Expression::TimeSec(v))
            }
            TokenKind::TimeMs(v) => {
                let v = *v;
                self.advance();
                Ok(Expression::TimeMs(v))
            }
            TokenKind::TimeFrames(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::TimeFrames(n))
            }
            TokenKind::Smpte(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::Smpte(s))
            }
            TokenKind::ColorLit(c) => {
                let c = c.clone();
                self.advance();
                Ok(Expression::ColorLit(c))
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expression::Ident(name))
            }
            _ => Err(VeacError::new(
                ErrorKind::ExpectedExpression,
                format!("expected expression, found {}", self.current().kind),
                Some(self.current().span),
            )),
        }
    }
}
