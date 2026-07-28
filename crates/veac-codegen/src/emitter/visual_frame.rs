use veac_plan::canonical::{FitMode, Vec2};
use veac_plan::EffectiveVisualProperties;

use super::{geometry, time, EmitContext};

#[derive(Clone, Copy)]
pub(super) enum FrameMode {
    FitSurface,
    AlreadySized,
    VisibleOverflow { basis_width: f64, basis_height: f64 },
}

#[derive(Clone, Copy)]
pub(super) struct SourceGeometry {
    pub mode: FrameMode,
    pub pivot_x: f64,
    pub pivot_y: f64,
}

impl SourceGeometry {
    pub fn framed(anchor: Vec2) -> Self {
        Self::new(FrameMode::FitSurface, anchor.x, anchor.y)
    }

    pub fn already_sized(anchor: Vec2) -> Self {
        Self::new(FrameMode::AlreadySized, anchor.x, anchor.y)
    }

    pub fn visible_overflow(
        surface: (u32, u32),
        origin: (f64, f64),
        basis: (f64, f64),
        anchor: Vec2,
    ) -> Self {
        Self::new(
            FrameMode::VisibleOverflow {
                basis_width: basis.0,
                basis_height: basis.1,
            },
            (origin.0 + anchor.x * basis.0) / f64::from(surface.0),
            (origin.1 + anchor.y * basis.1) / f64::from(surface.1),
        )
    }

    fn new(mode: FrameMode, pivot_x: f64, pivot_y: f64) -> Self {
        Self {
            mode,
            pivot_x,
            pivot_y,
        }
    }
}

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    input: &str,
    visual: &EffectiveVisualProperties,
    mode: FrameMode,
) -> String {
    let Some(frame) = visual.frame else {
        return input.to_owned();
    };
    let width = geometry::pixel_count(frame.width, context.canvas.width);
    let height = geometry::pixel_count(frame.height, context.canvas.height);
    match mode {
        FrameMode::AlreadySized => input.to_owned(),
        FrameMode::FitSurface => fit_surface(context, input, width, height, frame.fit),
        FrameMode::VisibleOverflow {
            basis_width,
            basis_height,
        } => fit_overflow(
            context,
            input,
            (width, height),
            (basis_width, basis_height),
            frame.fit,
        ),
    }
}

fn fit_surface(
    context: &mut EmitContext<'_>,
    input: &str,
    width: u32,
    height: u32,
    fit: FitMode,
) -> String {
    let filter = match fit {
        FitMode::Fill => format!("scale={width}:{height}"),
        FitMode::Contain => format!(
            "scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2:color=black@0,format=rgba"
        ),
        FitMode::Cover => format!(
            "scale={width}:{height}:force_original_aspect_ratio=increase,crop={width}:{height}"
        ),
    };
    context.graph.filter(&[input], filter, "framev")
}

fn fit_overflow(
    context: &mut EmitContext<'_>,
    input: &str,
    target: (u32, u32),
    basis: (f64, f64),
    fit: FitMode,
) -> String {
    let mut scale = (f64::from(target.0) / basis.0, f64::from(target.1) / basis.1);
    match fit {
        FitMode::Fill => {}
        FitMode::Contain => scale = (scale.0.min(scale.1), scale.0.min(scale.1)),
        FitMode::Cover => scale = (scale.0.max(scale.1), scale.0.max(scale.1)),
    }
    if scale.0 == 1.0 && scale.1 == 1.0 {
        return input.to_owned();
    }
    let filter = format!(
        "scale=w='max(1\\,round(iw*{}))':h='max(1\\,round(ih*{}))'",
        time::number(scale.0),
        time::number(scale.1)
    );
    context.graph.filter(&[input], filter, "overflowframev")
}
