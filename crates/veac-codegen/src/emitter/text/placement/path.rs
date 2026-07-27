use veac_plan::canonical::{LengthUnit, TextPath, TextPathAlignment};

use super::{placed, MeasuredPiece};
use crate::emitter::geometry;
use crate::emitter::text::error::TextError;
use crate::emitter::text::model::PlacedPiece;

const MAX_PATH_POINTS: usize = 256;
const EPSILON: f64 = 1e-6;

struct Segment {
    start: (f64, f64),
    end: (f64, f64),
    offset: f64,
    length: f64,
}

pub(super) fn place(
    lines: &[Vec<MeasuredPiece>],
    path: &TextPath,
    surface: (u32, u32),
) -> Result<Vec<PlacedPiece>, TextError> {
    if lines.len() != 1 {
        return Err(TextError::new(
            "TEXT_PATH_MULTILINE",
            "text path accepts exactly one logical line",
        ));
    }
    if !(2..=MAX_PATH_POINTS).contains(&path.points.len()) {
        return Err(TextError::new(
            "TEXT_PATH_POINT_LIMIT",
            format!("text path requires 2..={MAX_PATH_POINTS} points"),
        ));
    }
    let mut points: Vec<_> = path
        .points
        .iter()
        .map(|point| {
            (
                geometry::pixel_value(point.x, surface.0),
                geometry::pixel_value(point.y, surface.1),
            )
        })
        .collect();
    if path.reverse {
        points.reverse();
    }
    if points
        .iter()
        .flat_map(|point| [point.0, point.1])
        .any(|value| !value.is_finite())
    {
        return Err(TextError::new(
            "TEXT_PATH_INVALID",
            "text path contains a non-finite point",
        ));
    }
    let (segments, path_length) = segments(&points)?;
    let text_length: f64 = lines[0].iter().map(|piece| piece.advance).sum();
    let offset = path_offset(path, path_length);
    let start = match path.alignment {
        TextPathAlignment::Start => offset,
        TextPathAlignment::Center => offset - text_length / 2.0,
        TextPathAlignment::End => offset - text_length,
    };
    if !start.is_finite() || start < -EPSILON || start + text_length > path_length + EPSILON {
        return Err(TextError::new(
            "TEXT_PATH_BOUNDS",
            format!(
                "text range {start}..{} exceeds path length {path_length}",
                start + text_length
            ),
        ));
    }
    let mut cursor = start;
    let mut output = Vec::with_capacity(lines[0].len());
    for piece in &lines[0] {
        let distance = cursor + piece.advance / 2.0;
        let (x, y, rotation) = sample(&segments, distance);
        output.push(placed(piece, x, y, rotation));
        cursor += piece.advance;
    }
    Ok(output)
}

fn segments(points: &[(f64, f64)]) -> Result<(Vec<Segment>, f64), TextError> {
    let mut offset = 0.0;
    let mut output = Vec::with_capacity(points.len() - 1);
    for pair in points.windows(2) {
        let length = (pair[1].0 - pair[0].0).hypot(pair[1].1 - pair[0].1);
        if !length.is_finite() || length <= EPSILON {
            return Err(TextError::new(
                "TEXT_PATH_ZERO_LENGTH",
                "text path contains a zero-length segment",
            ));
        }
        output.push(Segment {
            start: pair[0],
            end: pair[1],
            offset,
            length,
        });
        offset += length;
    }
    Ok((output, offset))
}

fn path_offset(path: &TextPath, length: f64) -> f64 {
    match path.start_offset.unit {
        LengthUnit::Pixels => path.start_offset.value,
        LengthUnit::Normalized => path.start_offset.value * length,
        LengthUnit::Percent => path.start_offset.value * length / 100.0,
    }
}

fn sample(segments: &[Segment], distance: f64) -> (f64, f64, f64) {
    let segment = segments
        .iter()
        .find(|segment| distance <= segment.offset + segment.length)
        .unwrap_or_else(|| segments.last().expect("validated path has a segment"));
    let amount = ((distance - segment.offset) / segment.length).clamp(0.0, 1.0);
    let dx = segment.end.0 - segment.start.0;
    let dy = segment.end.1 - segment.start.1;
    (
        segment.start.0 + dx * amount,
        segment.start.1 + dy * amount,
        dy.atan2(dx).to_degrees(),
    )
}
