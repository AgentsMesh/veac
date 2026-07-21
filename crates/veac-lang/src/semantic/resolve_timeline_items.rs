/// Resolution of gap, freeze, and pip track item declarations.
use std::collections::HashMap;

use crate::ast::*;
use crate::error::{ErrorKind, VeacError};
use crate::ir::*;

use super::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    pub(crate) fn resolve_gap(
        &self,
        gap: &GapDecl,
        variables: &HashMap<String, Expression>,
        fps: u32,
    ) -> Result<IrGap, VeacError> {
        let mut duration_sec = 1.0;
        for attr in &gap.attributes {
            let val = self.resolve_expression(&attr.value, variables)?;
            if attr.key == "duration" {
                duration_sec = self.expr_to_seconds(val, fps, "duration")?;
            }
        }
        Ok(IrGap { duration_sec })
    }

    pub(crate) fn resolve_freeze(
        &self,
        freeze: &FreezeDecl,
        assets: &HashMap<&str, &IrAsset>,
        variables: &HashMap<String, Expression>,
        fps: u32,
    ) -> Result<IrFreeze, VeacError> {
        let asset = assets.get(freeze.asset_ref.as_str()).ok_or_else(|| {
            VeacError::new(
                ErrorKind::UndefinedAsset,
                format!("undefined asset `{}`", freeze.asset_ref),
                None,
            )
        })?;

        let mut at_sec = 0.0;
        let mut duration_sec = 3.0;
        for attr in &freeze.attributes {
            let val = self.resolve_expression(&attr.value, variables)?;
            match attr.key.as_str() {
                "at" => at_sec = self.expr_to_seconds(val, fps, "at")?,
                "duration" => duration_sec = self.expr_to_seconds(val, fps, "duration")?,
                _ => {}
            }
        }

        Ok(IrFreeze {
            asset_name: asset.name.clone(),
            asset_path: asset.path.clone(),
            at_sec,
            duration_sec,
        })
    }

    pub(crate) fn resolve_pip(
        &self,
        pip: &PipDecl,
        assets: &HashMap<&str, &IrAsset>,
        variables: &HashMap<String, Expression>,
        fps: u32,
    ) -> Result<IrPip, VeacError> {
        let asset = assets.get(pip.asset_ref.as_str()).ok_or_else(|| {
            VeacError::new(
                ErrorKind::UndefinedAsset,
                format!("undefined asset `{}`", pip.asset_ref),
                None,
            )
        })?;

        let mut from_sec = None;
        let mut to_sec = None;
        let mut at_sec = 0.0;
        let mut duration_sec = 5.0;
        let mut position = Position::BottomRight;
        let mut scale = 0.25;
        let mut zoom_in_sec = 0.0;
        let mut zoom_out_sec = 0.0;
        let mut fade_in_sec = 0.0;
        let mut fade_out_sec = 0.0;
        let mut margin_x = 0.0;
        let mut margin_y = 0.0;
        let mut width = 0.0;
        let mut height = 0.0;
        let mut style = CardStyle::default();
        let mut x = None;
        let mut y = None;

        for attr in &pip.attributes {
            let val = self.resolve_expression(&attr.value, variables)?;
            match attr.key.as_str() {
                "from" => from_sec = Some(self.expr_to_seconds(val, fps, "from")?),
                "to" => to_sec = Some(self.expr_to_seconds(val, fps, "to")?),
                "at" => at_sec = self.expr_to_seconds(val, fps, "at")?,
                "duration" => duration_sec = self.expr_to_seconds(val, fps, "duration")?,
                "position" => {
                    if let Expression::StringLit(s) = val {
                        position = Self::parse_position(s)?;
                    }
                }
                "scale" => scale = self.expr_to_f64(val, "scale")?,
                "zoom_in" => zoom_in_sec = self.expr_to_seconds(val, fps, "zoom_in")?,
                "zoom_out" => zoom_out_sec = self.expr_to_seconds(val, fps, "zoom_out")?,
                "fade_in" => fade_in_sec = self.expr_to_seconds(val, fps, "fade_in")?,
                "fade_out" => fade_out_sec = self.expr_to_seconds(val, fps, "fade_out")?,
                // `margin` sets both axes; `margin_x`/`margin_y` override per-axis (apply after).
                "margin" => {
                    let m = self.expr_to_f64(val, "margin")?;
                    margin_x = m;
                    margin_y = m;
                }
                "margin_x" => margin_x = self.expr_to_f64(val, "margin_x")?,
                "margin_y" => margin_y = self.expr_to_f64(val, "margin_y")?,
                "width" => width = self.expr_to_f64(val, "width")?,
                "height" => height = self.expr_to_f64(val, "height")?,
                "x" => x = Some(self.expr_to_f64(val, "x")?),
                "y" => y = Some(self.expr_to_f64(val, "y")?),
                other => {
                    self.apply_card_attr(other, val, &mut style)?;
                }
            }
        }

        Ok(IrPip {
            asset_name: asset.name.clone(),
            asset_path: asset.path.clone(),
            from_sec,
            to_sec,
            at_sec,
            duration_sec,
            position,
            scale,
            zoom_in_sec,
            zoom_out_sec,
            fade_in_sec,
            fade_out_sec,
            margin_x,
            margin_y,
            width,
            height,
            style,
            x,
            y,
        })
    }
}
