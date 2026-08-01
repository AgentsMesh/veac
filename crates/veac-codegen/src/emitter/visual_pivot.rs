use super::{time, EmitContext};

pub(super) fn center(
    context: &mut EmitContext<'_>,
    input: &str,
    pivot_x: f64,
    pivot_y: f64,
    prefix: &str,
) -> String {
    context.graph.filter(
        &[input],
        format!(
            "pad=w='2*max({x}*iw\\,(1-{x})*iw)':h='2*max({y}*ih\\,(1-{y})*ih)':x='ow/2-{x}*iw':y='oh/2-{y}*ih':color=black@0",
            x = time::number(pivot_x),
            y = time::number(pivot_y),
        ),
        prefix,
    )
}
