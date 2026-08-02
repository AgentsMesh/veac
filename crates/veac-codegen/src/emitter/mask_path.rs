use veac_plan::canonical::Vec2;

use super::time;

pub(super) fn signed_distance(
    points: &[Vec2],
    x: &str,
    y: &str,
    width: &str,
    height: &str,
) -> String {
    if points.len() < 3 {
        return "-1".to_owned();
    }
    let edges: Vec<_> = points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
        .collect();
    let crossings: Vec<_> = edges
        .iter()
        .filter_map(|(start, end)| crossing(*start, *end, x, y, width, height))
        .collect();
    let parity = format!("mod({}\\,2)", crossings.join("+"));
    let distance = edges
        .iter()
        .map(|(start, end)| edge_distance(*start, *end, x, y, width, height))
        .reduce(|left, right| format!("min({left}\\,{right})"))
        .expect("validated path has at least one edge");
    format!("({distance})*(2*eq({parity}\\,1)-1)")
}

fn crossing(start: Vec2, end: Vec2, x: &str, y: &str, width: &str, height: &str) -> Option<String> {
    if start.y == end.y {
        return None;
    }
    let (low, high) = if start.y < end.y {
        (start, end)
    } else {
        (end, start)
    };
    let x1 = vertex(low.x, width);
    let y1 = vertex(low.y, height);
    let dx = delta(high.x - low.x, width);
    let dy = delta(high.y - low.y, height);
    let y2 = vertex(high.y, height);
    Some(format!(
        "gte(({y})\\,{y1})*lt(({y})\\,{y2})*lt(({x})\\,{x1}+((({y})-{y1})*({dx})/({dy})))"
    ))
}

fn edge_distance(start: Vec2, end: Vec2, x: &str, y: &str, width: &str, height: &str) -> String {
    let x1 = vertex(start.x, width);
    let y1 = vertex(start.y, height);
    let dx = delta(end.x - start.x, width);
    let dy = delta(end.y - start.y, height);
    let length = format!("({dx})*({dx})+({dy})*({dy})");
    let projection =
        format!("clip(((({x})-{x1})*({dx})+(({y})-{y1})*({dy}))/max({length}\\,0.000001)\\,0\\,1)");
    format!("hypot(({x})-({x1}+({projection})*({dx}))\\,({y})-({y1}+({projection})*({dy})))")
}

fn vertex(value: f64, extent: &str) -> String {
    format!("({}-0.5)*({extent})", time::number(value))
}

fn delta(value: f64, extent: &str) -> String {
    format!("{}*({extent})", time::number(value))
}

#[cfg(test)]
mod tests;
