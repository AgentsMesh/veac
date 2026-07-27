use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn generator(&mut self, generator: &Generator, path: &str, item_id: &str) {
        match generator {
            Generator::Gradient { gradient } => self.gradient(gradient, path, item_id),
            Generator::Shape { shape } => self.shape(shape, path, item_id),
            Generator::Solid { .. } | Generator::Transparent | Generator::Silence => {}
        }
    }

    fn gradient(&mut self, gradient: &Gradient, path: &str, item_id: &str) {
        let (geometry_invalid, stops) = match gradient {
            Gradient::Linear { start, end, stops } => {
                (!point(*start) || !point(*end) || start == end, stops)
            }
            Gradient::Radial {
                center,
                radius,
                stops,
            } => (
                !point(*center) || !radius.is_finite() || *radius <= 0.0,
                stops,
            ),
        };
        let mut last = -1.0;
        let stops_invalid = !(2..=16).contains(&stops.len())
            || stops.iter().any(|stop| {
                let invalid = !stop.offset.is_finite()
                    || !(0.0..=1.0).contains(&stop.offset)
                    || stop.offset <= last;
                last = stop.offset;
                invalid
            });
        if geometry_invalid || stops_invalid {
            self.value_error("GENERATOR_GRADIENT", path, item_id);
        }
    }

    fn shape(&mut self, shape: &VectorShape, path: &str, item_id: &str) {
        if shape.fill.is_none() && shape.stroke.is_none() {
            self.value_error("GENERATOR_SHAPE_STYLE", path, item_id);
        }
        for paint in shape
            .fill
            .iter()
            .chain(shape.stroke.iter().map(|stroke| &stroke.paint))
        {
            if let Paint::Gradient { gradient } = paint {
                self.gradient(gradient, path, item_id);
            }
        }
        if shape
            .stroke
            .as_ref()
            .is_some_and(|stroke| !stroke.width_pixels.is_finite() || stroke.width_pixels <= 0.0)
        {
            self.value_error("GENERATOR_SHAPE_STYLE", path, item_id);
        }
        if invalid_geometry(&shape.geometry) {
            self.value_error("GENERATOR_SHAPE_GEOMETRY", path, item_id);
        }
    }
}

fn invalid_geometry(geometry: &VectorGeometry) -> bool {
    match geometry {
        VectorGeometry::Rectangle { bounds } | VectorGeometry::Ellipse { bounds } => !rect(*bounds),
        VectorGeometry::RoundedRectangle { bounds, radius } => {
            !rect(*bounds)
                || !radius.is_finite()
                || *radius <= 0.0
                || *radius > bounds.width.min(bounds.height) / 2.0
        }
        VectorGeometry::Polygon { points } => {
            !(3..=32).contains(&points.len())
                || points.iter().any(|value| !point(*value))
                || repeated_edge(points)
        }
        VectorGeometry::Path { commands } => invalid_path(commands),
    }
}

fn invalid_path(commands: &[PathCommand]) -> bool {
    if !(4..=34).contains(&commands.len())
        || !matches!(commands.first(), Some(PathCommand::MoveTo { .. }))
        || !matches!(commands.last(), Some(PathCommand::Close))
    {
        return true;
    }
    if commands
        .iter()
        .enumerate()
        .any(|(index, command)| match command {
            PathCommand::MoveTo { point: value } => index != 0 || !point(*value),
            PathCommand::LineTo { point: value } => !point(*value),
            PathCommand::Close => index + 1 != commands.len(),
        })
    {
        return true;
    }
    let points: Vec<_> = commands
        .iter()
        .filter_map(|command| match command {
            PathCommand::MoveTo { point } | PathCommand::LineTo { point } => Some(*point),
            PathCommand::Close => None,
        })
        .collect();
    repeated_edge(&points)
}

fn repeated_edge(points: &[Vec2]) -> bool {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .any(|(left, right)| left == right)
}

fn point(value: Vec2) -> bool {
    value.x.is_finite()
        && value.y.is_finite()
        && (0.0..=1.0).contains(&value.x)
        && (0.0..=1.0).contains(&value.y)
}

fn rect(value: Rect) -> bool {
    point(Vec2 {
        x: value.x,
        y: value.y,
    }) && value.width.is_finite()
        && value.height.is_finite()
        && value.width > 0.0
        && value.height > 0.0
        && value.x + value.width <= 1.0
        && value.y + value.height <= 1.0
}
