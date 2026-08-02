use veac_plan::canonical::VectorShape;
use veac_plan::ResolvedClip;

use super::{generated, generated_geometry, generated_gradient, EmitContext};

pub(super) fn render(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    shape: &VectorShape,
) -> String {
    let base = generated::high_precision_canvas(context, clip, "shapecanvasv");
    let fill_mask = shape.fill.as_ref().map_or_else(
        || "0".to_owned(),
        |_| generated_geometry::inside(&shape.geometry),
    );
    let stroke_mask = shape.stroke.as_ref().map_or_else(
        || "0".to_owned(),
        |stroke| generated_geometry::stroke(&shape.geometry, stroke.width_pixels),
    );
    let channels = (0..4)
        .map(|channel| {
            let fill = shape.fill.as_ref().map_or_else(
                || "0".to_owned(),
                |paint| generated_gradient::paint(paint, channel),
            );
            let stroke = shape.stroke.as_ref().map_or_else(
                || "0".to_owned(),
                |stroke| generated_gradient::paint(&stroke.paint, channel),
            );
            format!("if(gt({stroke_mask}\\,0)\\,{stroke}\\,if(gt({fill_mask}\\,0)\\,{fill}\\,0))")
        })
        .collect::<Vec<_>>();
    context.graph.filter(
        &[&base],
        format!(
            "geq=r='{}':g='{}':b='{}':a='{}'",
            channels[0], channels[1], channels[2], channels[3]
        ),
        "shapev",
    )
}
