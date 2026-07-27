use crate::*;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

pub(super) fn between<T>(
    keys: &[Keyframe<T>],
    start: RationalTime,
    end: RationalTime,
) -> Result<Interpolation, ()> {
    let first = keys.first().ok_or(())?;
    let last = keys.last().ok_or(())?;
    if end <= first.time || start >= last.time {
        return Ok(Interpolation::Hold);
    }
    let pair = keys
        .windows(2)
        .find(|pair| start >= pair[0].time && start < pair[1].time && end <= pair[1].time)
        .ok_or(())?;
    if start.timescale != pair[0].time.timescale || end.timescale != pair[0].time.timescale {
        return Err(());
    }
    let span = pair[1]
        .time
        .value
        .checked_sub(pair[0].time.value)
        .ok_or(())?;
    let offset_start = start.value.checked_sub(pair[0].time.value).ok_or(())?;
    let offset_end = end.value.checked_sub(pair[0].time.value).ok_or(())?;
    restrict(
        &pair[0].interpolation,
        offset_start as f64 / span as f64,
        offset_end as f64 / span as f64,
    )
}

fn restrict(value: &Interpolation, from: f64, to: f64) -> Result<Interpolation, ()> {
    if !(0.0..1.0).contains(&from) || !(from..=1.0).contains(&to) || from == to {
        return Err(());
    }
    if from == 0.0 && to == 1.0 {
        return Ok(value.clone());
    }
    if matches!(value, Interpolation::Hold | Interpolation::Linear) {
        return Ok(value.clone());
    }
    let controls = value.cubic_controls().ok_or(())?;
    let points = [
        Point { x: 0.0, y: 0.0 },
        Point {
            x: controls[0],
            y: controls[1],
        },
        Point {
            x: controls[2],
            y: controls[3],
        },
        Point { x: 1.0, y: 1.0 },
    ];
    let from = value.parameter_for_x(from).ok_or(())?;
    let to = value.parameter_for_x(to).ok_or(())?;
    normalize(crop(points, from, to))
}

fn crop(points: [Point; 4], from: f64, to: f64) -> [Point; 4] {
    let (_, right) = split(points, from);
    let local_to = (to - from) / (1.0 - from);
    split(right, local_to).0
}

fn split(points: [Point; 4], at: f64) -> ([Point; 4], [Point; 4]) {
    let a = lerp(points[0], points[1], at);
    let b = lerp(points[1], points[2], at);
    let c = lerp(points[2], points[3], at);
    let d = lerp(a, b, at);
    let e = lerp(b, c, at);
    let middle = lerp(d, e, at);
    ([points[0], a, d, middle], [middle, e, c, points[3]])
}

fn lerp(left: Point, right: Point, at: f64) -> Point {
    Point {
        x: left.x + (right.x - left.x) * at,
        y: left.y + (right.y - left.y) * at,
    }
}

fn normalize(points: [Point; 4]) -> Result<Interpolation, ()> {
    let width = points[3].x - points[0].x;
    let height = points[3].y - points[0].y;
    if width <= 0.0 || !width.is_finite() || !height.is_finite() {
        return Err(());
    }
    if height.abs() <= 1e-12 {
        let flat = points[1..3]
            .iter()
            .all(|point| (point.y - points[0].y).abs() <= 1e-12);
        return flat.then_some(Interpolation::Hold).ok_or(());
    }
    Ok(Interpolation::CubicBezier {
        x1: normalized(points[1].x, points[0].x, width)?,
        y1: (points[1].y - points[0].y) / height,
        x2: normalized(points[2].x, points[0].x, width)?,
        y2: (points[2].y - points[0].y) / height,
    })
}

fn normalized(value: f64, start: f64, extent: f64) -> Result<f64, ()> {
    let value = (value - start) / extent;
    if !value.is_finite() || !(-1e-12..=1.0 + 1e-12).contains(&value) {
        Err(())
    } else {
        Ok(value.clamp(0.0, 1.0))
    }
}
