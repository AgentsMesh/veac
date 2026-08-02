use veac_plan::canonical::{PathCommand, Rect, Vec2, VectorGeometry};

use super::time;

pub(super) fn inside(value: &VectorGeometry) -> String {
    match value {
        VectorGeometry::Rectangle { bounds } => rectangle(*bounds, 0.0),
        VectorGeometry::Ellipse { bounds } => ellipse(*bounds, 0.0),
        VectorGeometry::RoundedRectangle { bounds, radius } => rounded(*bounds, *radius, 0.0),
        VectorGeometry::Polygon { points } => polygon(points),
        VectorGeometry::Path { commands } => polygon(&path_points(commands)),
    }
}

pub(super) fn stroke(value: &VectorGeometry, width: f64) -> String {
    match value {
        VectorGeometry::Rectangle { bounds } => {
            let outer = rectangle(*bounds, 0.0);
            let inner = rectangle(*bounds, width);
            format!("({outer})*(1-({inner}))")
        }
        VectorGeometry::Ellipse { bounds } => {
            let outer = ellipse(*bounds, 0.0);
            let inner = ellipse(*bounds, width);
            format!("({outer})*(1-({inner}))")
        }
        VectorGeometry::RoundedRectangle { bounds, radius } => {
            let outer = rounded(*bounds, *radius, 0.0);
            let inner = rounded(*bounds, *radius, width);
            format!("({outer})*(1-({inner}))")
        }
        VectorGeometry::Polygon { points } => edge_stroke(points, width),
        VectorGeometry::Path { commands } => edge_stroke(&path_points(commands), width),
    }
}

fn rectangle(value: Rect, inset: f64) -> String {
    format!(
        "between(X\\,{}*W+{}\\,{}*W-{})*between(Y\\,{}*H+{}\\,{}*H-{})",
        n(value.x),
        n(inset),
        n(value.x + value.width),
        n(inset),
        n(value.y),
        n(inset),
        n(value.y + value.height),
        n(inset)
    )
}

fn ellipse(value: Rect, inset: f64) -> String {
    let cx = value.x + value.width / 2.0;
    let cy = value.y + value.height / 2.0;
    format!(
        "lte(pow((X-{}*W)/max(0.000001\\,{}*W-{})\\,2)+pow((Y-{}*H)/max(0.000001\\,{}*H-{})\\,2)\\,1)",
        n(cx),
        n(value.width / 2.0),
        n(inset),
        n(cy),
        n(value.height / 2.0),
        n(inset)
    )
}

fn rounded(value: Rect, radius: f64, inset: f64) -> String {
    let cx = value.x + value.width / 2.0;
    let cy = value.y + value.height / 2.0;
    let r = format!("max(0\\,{}*min(W\\,H)-{})", n(radius), n(inset));
    let half_width = format!("{}*W/2-{}", n(value.width), n(inset));
    let half_height = format!("{}*H/2-{}", n(value.height), n(inset));
    format!(
        "lte(hypot(max(abs(X-{}*W)-({half_width})+({r})\\,0)\\,max(abs(Y-{}*H)-({half_height})+({r})\\,0))\\,{r})",
        n(cx),
        n(cy)
    )
}

fn polygon(points: &[Vec2]) -> String {
    if points.len() < 3 {
        return "0".to_owned();
    }
    let crossings = edges(points)
        .map(|(left, right)| {
            let y1 = format!("{}*H", n(left.y));
            let y2 = format!("{}*H", n(right.y));
            let x1 = format!("{}*W", n(left.x));
            let x2 = format!("{}*W", n(right.x));
            format!(
                "if(eq({y1}\\,{y2})\\,0\\,gt(Y\\,min({y1}\\,{y2}))*lte(Y\\,max({y1}\\,{y2}))*lt(X\\,{x1}+(Y-{y1})*({x2}-{x1})/({y2}-{y1})))"
            )
        })
        .collect::<Vec<_>>()
        .join("+");
    format!("gt(mod({crossings}\\,2)\\,0)")
}

fn edge_stroke(points: &[Vec2], width: f64) -> String {
    if points.len() < 2 {
        return "0".to_owned();
    }
    let distances = edges(points)
        .map(|(left, right)| edge_distance(left, right))
        .collect::<Vec<_>>();
    let minimum = distances
        .into_iter()
        .reduce(|left, right| format!("min({left}\\,{right})"))
        .unwrap();
    format!("lte({minimum}\\,{})", n(width / 2.0))
}

fn edge_distance(left: Vec2, right: Vec2) -> String {
    let x1 = format!("{}*W", n(left.x));
    let y1 = format!("{}*H", n(left.y));
    let dx = format!("({}*W-{x1})", n(right.x));
    let dy = format!("({}*H-{y1})", n(right.y));
    let progress = format!("max(0\\,min(1\\,((X-{x1})*{dx}+(Y-{y1})*{dy})/({dx}*{dx}+{dy}*{dy})))");
    format!("hypot(X-({x1}+{dx}*({progress}))\\,Y-({y1}+{dy}*({progress})))")
}

fn edges(points: &[Vec2]) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
    points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
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

fn n(value: f64) -> String {
    time::number(value)
}
