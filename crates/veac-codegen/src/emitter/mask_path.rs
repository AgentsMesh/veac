use veac_plan::canonical::Vec2;

use super::time;

pub(super) fn signed_distance(points: &[Vec2], u: &str, v: &str, pixels: &str) -> String {
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
        .filter_map(|(start, end)| crossing(*start, *end, u, v))
        .collect();
    let parity = format!("mod({}\\,2)", crossings.join("+"));
    let distances: Vec<_> = edges
        .iter()
        .map(|(start, end)| edge_distance(*start, *end, u, v))
        .collect();
    let distance = distances
        .into_iter()
        .reduce(|left, right| format!("min({left}\\,{right})"))
        .expect("validated path has at least one edge");
    format!("if(eq({parity}\\,1)\\,({distance})*({pixels})\\,-({distance})*({pixels}))")
}

fn crossing(start: Vec2, end: Vec2, u: &str, v: &str) -> Option<String> {
    if start.y == end.y {
        return None;
    }
    let low = time::number(start.y.min(end.y));
    let high = time::number(start.y.max(end.y));
    let x = time::number(start.x);
    let dx = time::number(end.x - start.x);
    let dy = time::number(end.y - start.y);
    let y = time::number(start.y);
    Some(format!(
        "gte(({v})\\,{low})*lt(({v})\\,{high})*lt(({u})\\,{x}+((({v})-{y})*{dx}/{dy}))"
    ))
}

fn edge_distance(start: Vec2, end: Vec2, u: &str, v: &str) -> String {
    let x = time::number(start.x);
    let y = time::number(start.y);
    let dx = time::number(end.x - start.x);
    let dy = time::number(end.y - start.y);
    let length = time::number((end.x - start.x).powi(2) + (end.y - start.y).powi(2));
    let projection = format!("clip(((({u})-{x})*{dx}+(({v})-{y})*{dy})/{length}\\,0\\,1)");
    format!("hypot(({u})-({x}+({projection})*{dx})\\,({v})-({y}+({projection})*{dy}))")
}
