use veac_plan::canonical::{Animatable, Rect};

use super::{animation, time, EmitContext};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    input: &str,
    crop: &Option<Animatable<Rect>>,
) -> String {
    let Some(crop) = crop else {
        return input.to_owned();
    };
    match crop {
        Animatable::Constant { value } => static_crop(context, input, *value),
        Animatable::Keyframes { keyframes } => match keyframes.first() {
            Some(first) => animated_crop(context, input, crop, first.value),
            None => input.to_owned(),
        },
    }
}

fn static_crop(context: &mut EmitContext<'_>, input: &str, crop: Rect) -> String {
    context.graph.filter(
        &[input],
        format!(
            "crop=w='max(1\\,iw*{})':h='max(1\\,ih*{})':x='iw*{}':y='ih*{}'",
            time::number(crop.width),
            time::number(crop.height),
            time::number(crop.x),
            time::number(crop.y)
        ),
        "cropv",
    )
}

fn animated_crop(
    context: &mut EmitContext<'_>,
    input: &str,
    crop: &Animatable<Rect>,
    viewport: Rect,
) -> String {
    let width = animation::rect_width(crop, "t");
    let height = animation::rect_height(crop, "t");
    let scaled = context.graph.filter(
        &[input],
        format!(
            "scale=w='max(1\\,iw*{}/({width}))':h='max(1\\,ih*{}/({height}))':eval=frame",
            time::number(viewport.width),
            time::number(viewport.height)
        ),
        "cropzoomv",
    );
    let x = animation::rect_x(crop, "t");
    let y = animation::rect_y(crop, "t");
    context.graph.filter(
        &[&scaled],
        format!(
            "crop=w='max(1\\,iw*{})':h='max(1\\,ih*{})':x='iw*({x})':y='ih*({y})'",
            time::number(viewport.width),
            time::number(viewport.height)
        ),
        "cropv",
    )
}
