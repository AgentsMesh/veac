/// Resolution of clip declarations.
use std::collections::HashMap;

use crate::ast::*;
use crate::error::{ErrorKind, VeacError};
use crate::ir::*;

use super::SemanticAnalyzer;

/// Validate that a value falls within [min, max]. Returns the value on success.
fn validate_range(
    v: f64,
    min: f64,
    max: f64,
    clip_name: &str,
    field: &str,
) -> Result<f64, VeacError> {
    if !(min..=max).contains(&v) {
        return Err(VeacError::new(
            ErrorKind::InvalidValue,
            format!("clip `{clip_name}`: {field} {v} is out of range, must be {min} to {max}"),
            None,
        ));
    }
    Ok(v)
}

/// Validate that a value is strictly greater than min. Returns the value on success.
fn validate_gt(v: f64, min: f64, clip_name: &str, field: &str) -> Result<f64, VeacError> {
    if v <= min {
        return Err(VeacError::new(
            ErrorKind::InvalidValue,
            format!("clip `{clip_name}`: {field} must be > {min}, got {v}"),
            None,
        ));
    }
    Ok(v)
}

impl SemanticAnalyzer<'_> {
    pub(crate) fn resolve_clip(
        &self,
        clip: &ClipDecl,
        assets: &HashMap<&str, &IrAsset>,
        variables: &HashMap<String, Expression>,
        fps: u32,
    ) -> Result<IrClip, VeacError> {
        let asset = assets.get(clip.asset_ref.as_str()).ok_or_else(|| {
            VeacError::new(
                ErrorKind::UndefinedAsset,
                format!("undefined asset `{}`", clip.asset_ref),
                None,
            )
            .with_hint(format!(
                "define it with `asset {} = video(\"path\")`",
                clip.asset_ref
            ))
        })?;

        let mut ir = IrClip {
            asset_name: asset.name.clone(),
            asset_path: asset.path.clone(),
            asset_kind: asset.kind,
            ..Default::default()
        };
        let name = &clip.asset_ref;

        for attr in &clip.attributes {
            let val = self.resolve_expression(&attr.value, variables)?;
            match attr.key.as_str() {
                "from" => ir.from_sec = Some(self.expr_to_seconds(val, fps, "from")?),
                "to" => ir.to_sec = Some(self.expr_to_seconds(val, fps, "to")?),
                "duration" => ir.duration_sec = Some(self.expr_to_seconds(val, fps, "duration")?),
                "volume" => ir.volume = Some(self.expr_to_f64(val, "volume")?),
                "speed" => ir.speed = Some(self.expr_to_f64(val, "speed")?),
                "fade_in" => ir.fade_in_sec = Some(self.expr_to_seconds(val, fps, "fade_in")?),
                "fade_out" => ir.fade_out_sec = Some(self.expr_to_seconds(val, fps, "fade_out")?),
                "brightness" => ir.brightness = Some(self.expr_to_f64(val, "brightness")?),
                "contrast" => ir.contrast = Some(self.expr_to_f64(val, "contrast")?),
                "saturation" => ir.saturation = Some(self.expr_to_f64(val, "saturation")?),
                "zoom" => {
                    ir.zoom = Some(validate_range(
                        self.expr_to_f64(val, "zoom")?,
                        1.0,
                        10.0,
                        name,
                        "zoom",
                    )?)
                }
                "crop" => {
                    if let Expression::StringLit(s) = val {
                        if !(s.contains('x') && s.contains('+')) {
                            return Err(VeacError::new(
                                ErrorKind::InvalidValue,
                                format!("clip `{name}`: crop must be in WxH+X+Y format, got `{s}`"),
                                None,
                            )
                            .with_hint("example: crop = \"640x480+100+50\""));
                        }
                        ir.crop = Some(s.clone());
                    }
                }
                "blur" => {
                    ir.blur = Some(validate_range(
                        self.expr_to_f64(val, "blur")?,
                        0.0,
                        100.0,
                        name,
                        "blur",
                    )?)
                }
                "opacity" => {
                    ir.opacity = Some(validate_range(
                        self.expr_to_f64(val, "opacity")?,
                        0.0,
                        1.0,
                        name,
                        "opacity",
                    )?)
                }
                "rotate" => ir.rotate = Some(self.expr_to_f64(val, "rotate")?),
                "flip" => {
                    if let Expression::StringLit(s) = val {
                        match s.as_str() {
                            "horizontal" | "vertical" | "both" => ir.flip = Some(s.clone()),
                            _ => {
                                return Err(VeacError::new(
                                    ErrorKind::InvalidValue,
                                    format!("clip `{name}`: unknown flip mode `{s}`"),
                                    None,
                                )
                                .with_hint("supported: horizontal, vertical, both"))
                            }
                        }
                    }
                }
                "vignette" => {
                    ir.vignette = Some(validate_range(
                        self.expr_to_f64(val, "vignette")?,
                        0.0,
                        1.0,
                        name,
                        "vignette",
                    )?)
                }
                "grain" => {
                    ir.grain = Some(validate_range(
                        self.expr_to_f64(val, "grain")?,
                        0.0,
                        1.0,
                        name,
                        "grain",
                    )?)
                }
                "sharpen" => {
                    ir.sharpen = Some(validate_gt(
                        self.expr_to_f64(val, "sharpen")?,
                        0.0,
                        name,
                        "sharpen",
                    )?)
                }
                "pan_x" => {
                    ir.pan_x = Some(validate_range(
                        self.expr_to_f64(val, "pan_x")?,
                        -1.0,
                        1.0,
                        name,
                        "pan_x",
                    )?)
                }
                "pan_y" => {
                    ir.pan_y = Some(validate_range(
                        self.expr_to_f64(val, "pan_y")?,
                        -1.0,
                        1.0,
                        name,
                        "pan_y",
                    )?)
                }
                "reverse" => ir.reverse = Some(self.expr_to_bool(val, "reverse")?),
                "chromakey" => {
                    ir.chromakey = Some(match val {
                        Expression::StringLit(s) => s.clone(),
                        Expression::ColorLit(c) => format!("#{c}"),
                        _ => {
                            return Err(VeacError::new(
                                ErrorKind::TypeMismatch,
                                format!("clip `{name}`: chromakey expects a color or string"),
                                None,
                            )
                            .with_hint("example: chromakey = \"green\" or chromakey = #00FF00"))
                        }
                    });
                }
                "normalize" => ir.normalize = Some(self.expr_to_bool(val, "normalize")?),
                "loop" => {
                    let v = self.expr_to_u32(val, "loop")?;
                    if v == 0 {
                        return Err(VeacError::new(
                            ErrorKind::InvalidValue,
                            format!("clip `{name}`: loop count must be >= 1"),
                            None,
                        ));
                    }
                    ir.loop_count = Some(v);
                }
                "stabilize" => ir.stabilize = Some(self.expr_to_bool(val, "stabilize")?),
                _ => {}
            }
        }

        // Post-parse validations
        if let (Some(from), Some(to)) = (ir.from_sec, ir.to_sec) {
            if from >= to {
                return Err(VeacError::new(
                    ErrorKind::InvalidTimeRange,
                    format!("clip `{name}`: `from` ({from}s) must be less than `to` ({to}s)"),
                    None,
                ));
            }
        }
        if let Some(v) = ir.volume {
            validate_range(v, 0.0, 1.0, name, "volume")?;
        }
        if let Some(v) = ir.speed {
            if v <= 0.0 || v > 100.0 {
                return Err(VeacError::new(
                    ErrorKind::InvalidValue,
                    format!("clip `{name}`: speed {v} is out of range, must be > 0.0 and <= 100.0"),
                    None,
                ));
            }
        }
        if let Some(v) = ir.brightness {
            validate_range(v, -1.0, 1.0, name, "brightness")?;
        }
        if let Some(v) = ir.contrast {
            validate_range(v, 0.0, 3.0, name, "contrast")?;
        }
        if let Some(v) = ir.saturation {
            validate_range(v, 0.0, 3.0, name, "saturation")?;
        }

        Ok(ir)
    }
}
