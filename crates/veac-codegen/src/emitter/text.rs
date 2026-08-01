mod animation;
mod ass;
mod ass_fonts;
mod ass_tags;
pub(super) mod backend;
mod error;
mod escape;
mod font_face;
mod fonts;
mod layout;
mod lines;
mod model;
mod placement;
mod styled;
mod surface;
mod units;

use std::path::Path;

use veac_artifact::ExecutionBindings;
use veac_plan::{EffectiveVisualProperties, ResolvedClip, ResolvedFont, ResolvedText};

use super::{time, visual_pipeline, BackendFilterEscape, CodegenErrors, EmitContext};

pub(super) fn prepare(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    content: &ResolvedText,
    visual: &EffectiveVisualProperties,
) -> Result<visual_pipeline::PreparedLayer, CodegenErrors> {
    let rendered = backend::build(content, clip, context.bindings, context.canvas)
        .map_err(|failure| error::codegen(clip, failure))?;
    let duration = time::seconds(clip.record_range.duration);
    let canvas = transparent_canvas(
        context,
        rendered.surface.width,
        rendered.surface.height,
        &duration,
    );
    let font_token = context.filter_directory(
        rendered.font_directory,
        rendered.font_paths,
        BackendFilterEscape::Quoted,
    );
    let filter = ass::filter(&rendered.script, Path::new(&font_token));
    let premultiplied = context.graph.filter(&[&canvas], filter, "textassv");
    let corrected = correct_ass_alpha(context, &premultiplied);
    let source = context.graph.filter(
        &[&corrected],
        "unpremultiply=inplace=1:planes=7",
        "textstraightv",
    );
    visual_pipeline::apply_source(
        context,
        clip,
        visual,
        source,
        rendered.surface.source_geometry,
    )
}

fn correct_ass_alpha(context: &mut EmitContext<'_>, input: &str) -> String {
    // FFmpeg's ASS alpha path applies authored opacity to both coverage and the alpha plane.
    let (color_raw, alpha_raw) = context.graph.split(input, "textasssplitv");
    let color = super::rgb_planes::without_alpha(&mut context.graph, &color_raw, "textasscolorv");
    let alpha = context.graph.filter(
        &[&alpha_raw],
        concat!(
            "format=rgba64le,alphaextract,format=gray16le,",
            "geq=lum='sqrt(lum(X\\,Y)/65535)*65535'"
        ),
        "textassalphav",
    );
    super::alpha_merge::apply(context, &color, &alpha, "gbrap16le", "textassmergev")
}

fn transparent_canvas(
    context: &mut EmitContext<'_>,
    width: u32,
    height: u32,
    duration: &str,
) -> String {
    let rate = context.canvas.frame_rate;
    context.graph.source(
        format!(
            "color=c=black@0:s={width}x{height}:r={}/{}:d={duration},format=rgba",
            rate.numerator, rate.denominator
        ),
        "textcanvasv",
    )
}

pub(super) fn bound_ass_font_name(
    font: &ResolvedFont,
    bindings: &ExecutionBindings,
) -> Result<String, String> {
    let resource = bindings
        .input(&font.input_id)
        .and_then(|binding| binding.resource())
        .ok_or_else(|| format!("font {} has no resource binding", font.input_id))?;
    let bytes = resource
        .read_verified_bounded(veac_artifact::MAX_IN_MEMORY_ARTIFACT_BYTES)
        .map_err(|error| format!("cannot verify font {}: {error}", font.input_id))?;
    let mut database = cosmic_text::fontdb::Database::new();
    font_face::add(&mut database, bytes, font.face_index, 0)
        .map(|face| face.ass_name)
        .map_err(|error| format!("{}: {}", error.code, error.message))
}
