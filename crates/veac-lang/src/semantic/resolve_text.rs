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
        let mut margin = None;
        let mut shadow: Option<Shadow> = None;
        let mut outline: Option<Outline> = None;
        let mut x = None;
        let mut y = None;
        // Text shadow is a hard drawtext shadow (no blur); tighter default offset than a card's.
        let text_shadow = || Shadow {
            blur: 0.0,
            opacity: 1.0,
            dx: 2.0,
            dy: 2.0,
            color: "black".into(),
        };

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
                "margin" => margin = Some(self.expr_to_u32(val, "margin")?),
                "shadow" => {
                    if self.expr_to_bool(val, "shadow")? {
                        shadow.get_or_insert_with(text_shadow);
                    } else {
                        shadow = None;
                    }
                }
                "shadow_x" => {
                    shadow.get_or_insert_with(text_shadow).dx = self.expr_to_f64(val, "shadow_x")?
                }
                "shadow_y" => {
                    shadow.get_or_insert_with(text_shadow).dy = self.expr_to_f64(val, "shadow_y")?
                }
                "shadow_color" => {
                    shadow.get_or_insert_with(text_shadow).color = self.expr_to_color(val)
                }
                "outline" => {
                    outline
                        .get_or_insert_with(|| Outline {
                            width: 0,
                            color: "black".into(),
                        })
                        .width = self.expr_to_u32(val, "outline")?
                }
                "outline_color" => {
                    outline
                        .get_or_insert_with(|| Outline {
                            width: 2,
                            color: "black".into(),
                        })
                        .color = self.expr_to_color(val)
                }
                "x" => x = Some(self.expr_to_f64(val, "x")?),
                "y" => y = Some(self.expr_to_f64(val, "y")?),
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
            margin,
            shadow,
            outline,
            x,
            y,
        })
    }
}
