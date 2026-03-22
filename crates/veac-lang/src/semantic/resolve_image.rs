/// Resolution of image overlay declarations.
use std::collections::HashMap;

use crate::ast::*;
use crate::error::{ErrorKind, VeacError};
use crate::ir::*;

use super::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    pub(crate) fn resolve_image_overlay(
        &self,
        decl: &ImageOverlayDecl,
        assets: &HashMap<&str, &IrAsset>,
        variables: &HashMap<String, Expression>,
        fps: u32,
    ) -> Result<IrImageOverlay, VeacError> {
        let asset = assets.get(decl.asset_ref.as_str()).ok_or_else(|| {
            VeacError::new(
                ErrorKind::UndefinedAsset,
                format!("undefined asset `{}`", decl.asset_ref),
                None,
            )
            .with_hint(format!(
                "define it with `asset {} = image(\"path\")`",
                decl.asset_ref
            ))
        })?;

        let mut at_sec = 0.0;
        let mut duration_sec = 5.0;
        let mut position = Position::TopRight;
        let mut scale = None;
        let mut opacity = None;

        for attr in &decl.attributes {
            let val = self.resolve_expression(&attr.value, variables)?;
            match attr.key.as_str() {
                "at" => at_sec = self.expr_to_seconds(val, fps, "at")?,
                "duration" => duration_sec = self.expr_to_seconds(val, fps, "duration")?,
                "position" => {
                    if let Expression::StringLit(s) = val {
                        position = Self::parse_position(s)?;
                    }
                }
                "scale" => {
                    let v = self.expr_to_f64(val, "scale")?;
                    if !(0.0..=10.0).contains(&v) {
                        return Err(VeacError::new(
                            ErrorKind::InvalidValue,
                            format!("scale {v} is out of range, must be 0.0 to 10.0"),
                            None,
                        ));
                    }
                    scale = Some(v);
                }
                "opacity" => {
                    let v = self.expr_to_f64(val, "opacity")?;
                    if !(0.0..=1.0).contains(&v) {
                        return Err(VeacError::new(
                            ErrorKind::InvalidValue,
                            format!("opacity {v} is out of range, must be 0.0 to 1.0"),
                            None,
                        ));
                    }
                    opacity = Some(v);
                }
                _ => {}
            }
        }

        Ok(IrImageOverlay {
            asset_name: asset.name.clone(),
            asset_path: asset.path.clone(),
            at_sec,
            duration_sec,
            position,
            scale,
            opacity,
        })
    }
}
