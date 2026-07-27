use veac_plan::canonical::{
    Generator, Gradient, Paint, PathCommand, Rect, Vec2, VectorGeometry, VectorShape,
};
use veac_plan::{ResolvedClipSource, ResolvedSequence};

use super::Check;

pub(super) fn validate(check: &mut Check, sequence: &ResolvedSequence) {
    for clip in sequence.tracks.iter().flat_map(|track| &track.clips) {
        let ResolvedClipSource::Generated { generator } = &clip.source else {
            continue;
        };
        let valid = match generator {
            Generator::Gradient { gradient } => gradient_valid(gradient),
            Generator::Shape { shape } => shape_valid(shape),
            Generator::Solid { .. } | Generator::Transparent | Generator::Silence => true,
        };
        if !valid {
            check.push(
                "PLAN_GENERATOR_INVALID",
                Some(clip.id.to_string()),
                "generated gradient or vector shape is malformed",
            );
        }
    }
}

fn gradient_valid(value: &Gradient) -> bool {
    let (geometry, stops) = match value {
        Gradient::Linear { start, end, stops } => {
            (point(*start) && point(*end) && start != end, stops)
        }
        Gradient::Radial {
            center,
            radius,
            stops,
        } => (point(*center) && radius.is_finite() && *radius > 0.0, stops),
    };
    let mut previous = -1.0;
    geometry
        && (2..=16).contains(&stops.len())
        && stops.iter().all(|stop| {
            let valid = stop.offset.is_finite()
                && (0.0..=1.0).contains(&stop.offset)
                && stop.offset > previous;
            previous = stop.offset;
            valid
        })
}

fn shape_valid(value: &VectorShape) -> bool {
    let paints = value
        .fill
        .iter()
        .chain(value.stroke.iter().map(|stroke| &stroke.paint));
    let paint_valid = paints.into_iter().all(|paint| match paint {
        Paint::Solid { .. } => true,
        Paint::Gradient { gradient } => gradient_valid(gradient),
    });
    let stroke_valid = value
        .stroke
        .as_ref()
        .is_none_or(|stroke| stroke.width_pixels.is_finite() && stroke.width_pixels > 0.0);
    (value.fill.is_some() || value.stroke.is_some())
        && paint_valid
        && stroke_valid
        && geometry_valid(&value.geometry)
}

fn geometry_valid(value: &VectorGeometry) -> bool {
    match value {
        VectorGeometry::Rectangle { bounds } | VectorGeometry::Ellipse { bounds } => rect(*bounds),
        VectorGeometry::RoundedRectangle { bounds, radius } => {
            rect(*bounds)
                && radius.is_finite()
                && *radius > 0.0
                && *radius <= bounds.width.min(bounds.height) / 2.0
        }
        VectorGeometry::Polygon { points } => {
            (3..=32).contains(&points.len())
                && points.iter().all(|p| point(*p))
                && !repeated_edge(points)
        }
        VectorGeometry::Path { commands } => {
            (4..=34).contains(&commands.len())
                && matches!(commands.first(), Some(PathCommand::MoveTo { .. }))
                && matches!(commands.last(), Some(PathCommand::Close))
                && commands
                    .iter()
                    .enumerate()
                    .all(|(index, command)| match command {
                        PathCommand::MoveTo { point: value } => index == 0 && point(*value),
                        PathCommand::LineTo { point: value } => point(*value),
                        PathCommand::Close => index + 1 == commands.len(),
                    })
                && !repeated_edge(&path_points(commands))
        }
    }
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

fn path_points(commands: &[PathCommand]) -> Vec<Vec2> {
    commands
        .iter()
        .filter_map(|command| match command {
            PathCommand::MoveTo { point } | PathCommand::LineTo { point } => Some(*point),
            PathCommand::Close => None,
        })
        .collect()
}

fn repeated_edge(points: &[Vec2]) -> bool {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .any(|(left, right)| left == right)
}
