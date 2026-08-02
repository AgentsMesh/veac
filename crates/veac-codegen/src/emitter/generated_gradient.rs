use veac_plan::canonical::{Color, Gradient, Paint};
use veac_plan::ResolvedClip;

use super::{generated, time, EmitContext};

pub(super) fn render(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    gradient: &Gradient,
) -> String {
    let base = generated::high_precision_canvas(context, clip, "gradientcanvasv");
    let channels = (0..4)
        .map(|channel| channel_expression(gradient, channel))
        .collect::<Vec<_>>();
    context.graph.filter(
        &[&base],
        format!(
            "geq=r='{}':g='{}':b='{}':a='{}'",
            channels[0], channels[1], channels[2], channels[3]
        ),
        "gradientv",
    )
}

pub(super) fn paint(value: &Paint, channel: usize) -> String {
    match value {
        Paint::Solid { color } => time::number(component(*color, channel)),
        Paint::Gradient { gradient } => channel_expression(gradient, channel),
    }
}

fn channel_expression(gradient: &Gradient, channel: usize) -> String {
    let (position, stops) = match gradient {
        Gradient::Linear { start, end, stops } => {
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let denominator = dx * dx + dy * dy;
            (
                format!(
                    "(((X/max(1\\,W-1))-{})*{}+((Y/max(1\\,H-1))-{})*{})/{}",
                    time::number(start.x),
                    time::number(dx),
                    time::number(start.y),
                    time::number(dy),
                    time::number(denominator)
                ),
                stops,
            )
        }
        Gradient::Radial {
            center,
            radius,
            stops,
        } => (
            format!(
                "hypot((X/max(1\\,W-1))-{}\\,(Y/max(1\\,H-1))-{})/{}",
                time::number(center.x),
                time::number(center.y),
                time::number(*radius)
            ),
            stops,
        ),
    };
    let first = time::number(component(stops[0].color, channel));
    let mut result = time::number(component(stops.last().unwrap().color, channel));
    for pair in stops.windows(2).rev() {
        let left = pair[0];
        let right = pair[1];
        let progress = format!(
            "max(0\\,min(1\\,(({})-{})/{}))",
            position,
            time::number(left.offset),
            time::number(right.offset - left.offset)
        );
        let left_value = time::number(component(left.color, channel));
        let right_value = time::number(component(right.color, channel));
        let value = format!("{left_value}+({right_value}-{left_value})*({progress})");
        result = format!(
            "if(lte({}\\,{})\\,{value}\\,{result})",
            position,
            time::number(right.offset)
        );
    }
    format!(
        "if(lte({}\\,{})\\,{first}\\,{result})",
        position,
        time::number(stops[0].offset)
    )
}

fn component(color: Color, channel: usize) -> f64 {
    f64::from(match channel {
        0 => color.red,
        1 => color.green,
        2 => color.blue,
        _ => color.alpha,
    }) * 257.0
}
