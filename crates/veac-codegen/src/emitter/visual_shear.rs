use veac_plan::canonical::Vec2;

use super::{time, visual_pivot, EmitContext};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    input: &str,
    shear: Vec2,
    pivot_x: f64,
    pivot_y: f64,
) -> (String, f64, f64) {
    if shear.x == 0.0 && shear.y == 0.0 {
        return (input.to_owned(), pivot_x, pivot_y);
    }
    let alpha = context
        .graph
        .filter(&[input], "format=gbrap16le", "shearprecisionv");
    let centered = visual_pivot::center(context, &alpha, pivot_x, pivot_y, "shearpivotv");
    let x_extent = time::number(shear.x.abs());
    let y_extent = time::number(shear.y.abs());
    let expanded = context.graph.filter(
        &[&centered],
        format!(
            "pad=w='ceil(iw+{x_extent}*ih)':h='ceil(ih+{y_extent}*iw)':\
             x='(ow-iw)/2':y='(oh-ih)/2':color=black@0"
        ),
        "shearboundv",
    );
    let output = context.graph.filter(
        &[&expanded],
        format!(
            "shear=shx={}:shy={}:fillcolor=black@0:interp=bilinear",
            time::number(shear.x),
            time::number(shear.y),
        ),
        "shearv",
    );
    (output, 0.5, 0.5)
}
