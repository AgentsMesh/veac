/// Resolution of text overlay declarations.
use std::collections::HashMap;

use crate::ast::*;
use crate::error::VeacError;
use crate::ir::*;

use super::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    pub(crate) fn resolve_text_overlay(
        &self,
        text: &TextOverlayDecl,
        variables: &HashMap<String, Expression>,
        fps: u32,
    ) -> Result<IrTextOverlay, VeacError> {
        let mut at_sec = 0.0;
        let mut duration_sec = 5.0;
        let mut font = "Arial".to_string();
        let mut size: u32 = 24;
        let mut color = "FFFFFF".to_string();
        let mut position = Position::Center;
        let mut fade_in_sec = None;
        let mut fade_out_sec = None;
        let mut background = None;
        let mut background_padding = None;

        for attr in &text.attributes {
            let val = self.resolve_expression(&attr.value, variables)?;
            match attr.key.as_str() {
                "at" => at_sec = self.expr_to_seconds(val, fps, "at")?,
                "duration" => duration_sec = self.expr_to_seconds(val, fps, "duration")?,
                "font" => {
                    if let Expression::StringLit(s) = val {
                        font = s.clone();
                    }
                }
                "size" => size = self.expr_to_u32(val, "size")?,
                "color" => match val {
                    Expression::ColorLit(c) => color = c.clone(),
                    Expression::StringLit(c) => color = c.trim_start_matches('#').to_string(),
                    _ => {}
                },
                "position" => {
                    if let Expression::StringLit(s) = val {
                        position = Self::parse_position(s)?;
                    }
                }
                "fade_in" => fade_in_sec = Some(self.expr_to_seconds(val, fps, "fade_in")?),
                "fade_out" => fade_out_sec = Some(self.expr_to_seconds(val, fps, "fade_out")?),
                "background" => {
                    if let Expression::StringLit(s) = val {
                        background = Some(s.clone());
                    }
                }
                "background_padding" => {
                    background_padding = Some(self.expr_to_u32(val, "background_padding")?);
                }
                _ => {}
            }
        }

        Ok(IrTextOverlay {
            content: text.content.clone(),
            at_sec,
            duration_sec,
            font,
            size,
            color,
            position,
            fade_in_sec,
            fade_out_sec,
            resolved_font_path: None,
            background,
            background_padding,
        })
    }
}
