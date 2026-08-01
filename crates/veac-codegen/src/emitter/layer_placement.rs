use super::{alpha_merge, rgb_planes, Canvas, EmitContext};

pub(super) fn place_animated_canvas(
    context: &mut EmitContext<'_>,
    canvas: &str,
    source: &str,
    x: &str,
    y: &str,
    start: &str,
    end: &str,
) -> String {
    let (canvas_color, canvas_alpha) = context.graph.split(canvas, "placementcanvassplitv");
    let (source_color, source_alpha) = context.graph.split(source, "placementsourcesplitv");
    let canvas_color =
        rgb_planes::without_alpha(&mut context.graph, &canvas_color, "placementcolorbasev");
    let canvas_alpha =
        rgb_planes::without_alpha(&mut context.graph, &canvas_alpha, "placementalphabasev");
    let source_color =
        rgb_planes::without_alpha(&mut context.graph, &source_color, "placementcolorsourcev");
    let source_alpha = context.graph.filter(
        &[&source_alpha],
        "format=rgba64le,alphaextract,format=gray16le,format=gbrp16le",
        "placementalphasourcev",
    );
    let color = overlay(context, &canvas_color, &source_color, x, y, start, end);
    let alpha = overlay(context, &canvas_alpha, &source_alpha, x, y, start, end);
    let alpha = context.graph.filter(
        &[&alpha],
        "extractplanes=g,format=gray16le",
        "placementalphav",
    );
    alpha_merge::apply(context, &color, &alpha, "gbrap16le", "placementv")
}

fn overlay(
    context: &mut EmitContext<'_>,
    canvas: &str,
    source: &str,
    x: &str,
    y: &str,
    start: &str,
    end: &str,
) -> String {
    let filter = format!(
        concat!(
            "overlay=x='{}':y='{}':eof_action=pass:repeatlast=0:shortest=0:",
            "format=auto:alpha=straight:enable='gte(t,{})*lt(t,{})'"
        ),
        x, y, start, end,
    );
    let placed = context
        .graph
        .filter(&[canvas, source], filter, "placement10v");
    context
        .graph
        .filter(&[&placed], "format=gbrp16le", "placement16v")
}

pub(super) fn place_on_canvas(
    context: &mut EmitContext<'_>,
    source: &str,
    x: &str,
    y: &str,
    canvas: Canvas,
    extent: (u128, u128),
) -> String {
    let filter = format!(
        concat!(
            "format=gbrap16le,",
            "pad=w='{mw}*2+{w}':h='{mh}*2+{h}':",
            r"x='{mw}+clip(({x})\,-iw\,{w})':y='{mh}+clip(({y})\,-ih\,{h})':",
            "color=black@0:eval=frame,",
            "crop=w={w}:h={h}:x={mw}:y={mh}"
        ),
        w = canvas.width,
        h = canvas.height,
        x = x,
        y = y,
        mw = extent.0,
        mh = extent.1,
    );
    context.graph.filter(&[source], filter, "layerplacedv")
}
