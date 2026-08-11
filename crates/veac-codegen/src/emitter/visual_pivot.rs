use super::{time, EmitContext};

pub(super) fn center(
    context: &mut EmitContext<'_>,
    input: &str,
    pivot_x: f64,
    pivot_y: f64,
    prefix: &str,
) -> String {
    if pivot_x == 0.5 && pivot_y == 0.5 {
        return context.graph.filter(&[input], "format=gbrap16le", prefix);
    }
    context.graph.filter(
        &[input],
        format!(
            "format=gbrap16le,pad=w='2*max({x}*iw\\,(1-{x})*iw)':h='2*max({y}*ih\\,(1-{y})*ih)':x='ow/2-{x}*iw':y='oh/2-{y}*ih':color=black@0",
            x = time::number(pivot_x),
            y = time::number(pivot_y),
        ),
        prefix,
    )
}

pub(super) fn stabilize(
    context: &mut EmitContext<'_>,
    input: &str,
    pivot_x: f64,
    pivot_y: f64,
    extent: (u128, u128),
) -> String {
    let (width, height) = extent;
    let x = time::number(pivot_x);
    let y = time::number(pivot_y);
    context.graph.filter(
        &[input],
        format!(
            "format=gbrap16le,pad=w={width}:h={height}:x='{x}*{width}-{x}*iw':\
             y='{y}*{height}-{y}*ih':color=black@0:eval=frame"
        ),
        "scaleextentv",
    )
}
