/// Project resolution and value conversion helpers.
use std::collections::HashMap;

use crate::ast::*;
use crate::error::{ErrorKind, VeacError};
use crate::ir::*;

use super::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    pub(crate) fn resolve_project(
        &self,
        decl: &ProjectDecl,
        variables: &HashMap<String, Expression>,
    ) -> Result<IrProject, VeacError> {
        let mut width: u32 = 1920;
        let mut height: u32 = 1080;
        let mut fps: u32 = 30;
        let mut format = OutputFormat::Mp4;
        let mut codec = Codec::H264;
        let mut quality = Quality::High;
        let mut fit = FitMode::Fill;

        for attr in &decl.attributes {
            let val = self.resolve_expression(&attr.value, variables)?;
            match attr.key.as_str() {
                "resolution" => {
                    (width, height) = Self::parse_resolution(val)?;
                }
                "fps" => fps = self.expr_to_u32(val, "fps")?,
                "format" => format = Self::parse_format(val)?,
                "codec" => codec = Self::parse_codec(val)?,
                "quality" => quality = Self::parse_quality(val)?,
                "fit" => {
                    if let Expression::StringLit(s) = val {
                        fit = match s.as_str() {
                            "fill" => FitMode::Fill,
                            "letterbox" => FitMode::Letterbox,
                            "crop" => FitMode::Crop,
                            _ => {
                                return Err(VeacError::new(
                                    ErrorKind::InvalidValue,
                                    format!("unknown fit mode `{s}`"),
                                    None,
                                )
                                .with_hint("supported: fill, letterbox, crop"))
                            }
                        };
                    }
                }
                _ => {}
            }
        }

        Ok(IrProject {
            name: decl.name.clone(),
            width,
            height,
            fps,
            format,
            codec,
            quality,
            fit,
        })
    }

    // --- Value conversion helpers ---

    pub(crate) fn expr_to_seconds(
        &self,
        expr: &Expression,
        fps: u32,
        field: &str,
    ) -> Result<f64, VeacError> {
        match expr {
            Expression::TimeSec(v) => Ok(*v),
            Expression::TimeMs(v) => Ok(*v / 1000.0),
            Expression::TimeFrames(n) => Ok(*n as f64 / fps as f64),
            Expression::Smpte(s) => Self::parse_smpte(s, fps),
            Expression::IntLit(n) => Ok(*n as f64),
            Expression::FloatLit(n) => Ok(*n),
            _ => Err(VeacError::new(
                ErrorKind::TypeMismatch,
                format!("`{field}` expects a time value (e.g. 3.5s, 500ms, 84f)"),
                None,
            )),
        }
    }

    pub(crate) fn expr_to_f64(&self, expr: &Expression, field: &str) -> Result<f64, VeacError> {
        match expr {
            Expression::FloatLit(n) => Ok(*n),
            Expression::IntLit(n) => Ok(*n as f64),
            _ => Err(VeacError::new(
                ErrorKind::TypeMismatch,
                format!("`{field}` expects a number"),
                None,
            )),
        }
    }

    pub(crate) fn expr_to_bool(&self, expr: &Expression, field: &str) -> Result<bool, VeacError> {
        match expr {
            Expression::BoolLit(b) => Ok(*b),
            _ => Err(VeacError::new(
                ErrorKind::TypeMismatch,
                format!("`{field}` expects a boolean"),
                None,
            )),
        }
    }

    pub(crate) fn expr_to_u32(&self, expr: &Expression, field: &str) -> Result<u32, VeacError> {
        match expr {
            Expression::IntLit(n) if *n >= 0 => Ok(*n as u32),
            Expression::IntLit(n) => Err(VeacError::new(
                ErrorKind::InvalidValue,
                format!("`{field}` must be positive, got {n}"),
                None,
            )),
            _ => Err(VeacError::new(
                ErrorKind::TypeMismatch,
                format!("`{field}` expects an integer"),
                None,
            )),
        }
    }

    pub(crate) fn parse_smpte(s: &str, fps: u32) -> Result<f64, VeacError> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 {
            return Err(VeacError::new(
                ErrorKind::InvalidTimeLiteral,
                format!("invalid SMPTE timecode `{s}`"),
                None,
            ));
        }
        let h: f64 = parts[0].parse().unwrap_or(0.0);
        let m: f64 = parts[1].parse().unwrap_or(0.0);
        let sec: f64 = parts[2].parse().unwrap_or(0.0);
        let f: f64 = parts[3].parse().unwrap_or(0.0);
        Ok(h * 3600.0 + m * 60.0 + sec + f / fps as f64)
    }

    pub(crate) fn parse_resolution(val: &Expression) -> Result<(u32, u32), VeacError> {
        if let Expression::StringLit(s) = val {
            let parts: Vec<&str> = s.split('x').collect();
            if parts.len() != 2 {
                return Err(VeacError::new(
                    ErrorKind::InvalidValue,
                    format!("invalid resolution `{s}`, expected `WIDTHxHEIGHT`"),
                    None,
                ));
            }
            let w = parts[0].parse().map_err(|_| {
                VeacError::new(
                    ErrorKind::InvalidValue,
                    format!("invalid width in `{s}`"),
                    None,
                )
            })?;
            let h = parts[1].parse().map_err(|_| {
                VeacError::new(
                    ErrorKind::InvalidValue,
                    format!("invalid height in `{s}`"),
                    None,
                )
            })?;
            Ok((w, h))
        } else {
            Err(VeacError::new(
                ErrorKind::TypeMismatch,
                "resolution must be a string like \"1920x1080\"",
                None,
            ))
        }
    }

    pub(crate) fn parse_format(val: &Expression) -> Result<OutputFormat, VeacError> {
        if let Expression::StringLit(s) = val {
            match s.as_str() {
                "mp4" => Ok(OutputFormat::Mp4),
                "mkv" => Ok(OutputFormat::Mkv),
                "webm" => Ok(OutputFormat::Webm),
                "mov" => Ok(OutputFormat::Mov),
                _ => Err(VeacError::new(
                    ErrorKind::InvalidValue,
                    format!("unknown format `{s}`"),
                    None,
                )
                .with_hint("supported: mp4, mkv, webm, mov")),
            }
        } else {
            Err(VeacError::new(
                ErrorKind::TypeMismatch,
                "format must be a string",
                None,
            ))
        }
    }

    pub(crate) fn parse_codec(val: &Expression) -> Result<Codec, VeacError> {
        if let Expression::StringLit(s) = val {
            match s.as_str() {
                "h264" => Ok(Codec::H264),
                "h265" => Ok(Codec::H265),
                "vp9" => Ok(Codec::Vp9),
                "av1" => Ok(Codec::Av1),
                _ => Err(VeacError::new(
                    ErrorKind::InvalidValue,
                    format!("unknown codec `{s}`"),
                    None,
                )
                .with_hint("supported: h264, h265, vp9, av1")),
            }
        } else {
            Err(VeacError::new(
                ErrorKind::TypeMismatch,
                "codec must be a string",
                None,
            ))
        }
    }

    pub(crate) fn parse_quality(val: &Expression) -> Result<Quality, VeacError> {
        if let Expression::StringLit(s) = val {
            match s.as_str() {
                "low" => Ok(Quality::Low),
                "medium" => Ok(Quality::Medium),
                "high" => Ok(Quality::High),
                "lossless" => Ok(Quality::Lossless),
                _ => Err(VeacError::new(
                    ErrorKind::InvalidValue,
                    format!("unknown quality `{s}`"),
                    None,
                )
                .with_hint("supported: low, medium, high, lossless")),
            }
        } else {
            Err(VeacError::new(
                ErrorKind::TypeMismatch,
                "quality must be a string",
                None,
            ))
        }
    }

    pub(crate) fn parse_position(s: &str) -> Result<Position, VeacError> {
        match s {
            "center" => Ok(Position::Center),
            "top-left" => Ok(Position::TopLeft),
            "top-right" => Ok(Position::TopRight),
            "bottom-left" => Ok(Position::BottomLeft),
            "bottom-right" => Ok(Position::BottomRight),
            "top" => Ok(Position::Top),
            "bottom" => Ok(Position::Bottom),
            "left" => Ok(Position::Left),
            "right" => Ok(Position::Right),
            _ => Err(VeacError::new(ErrorKind::InvalidValue, format!("unknown position `{s}`"), None)
                .with_hint("use: center, top-left, top-right, bottom-left, bottom-right, top, bottom, left, right")),
        }
    }
}
