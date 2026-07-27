mod animation;
mod ass;
mod ass_fonts;
mod ass_tags;
mod backend;
mod error;
mod escape;
mod font_face;
mod fonts;
mod layout;
mod lines;
mod model;
mod placement;
mod styled;
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
    let canvas = transparent_canvas(context, rendered.width, rendered.height, &duration);
    let font_token = context.filter_directory(
        rendered.font_directory,
        rendered.font_paths,
        BackendFilterEscape::Quoted,
    );
    let filter = ass::filter(&rendered.script, Path::new(&font_token));
    let source = context.graph.filter(&[&canvas], filter, "textassv");
    Ok(if content.style.layout.box_width_pixels.is_some() {
        visual_pipeline::apply(context, clip, visual, source)?
    } else {
        visual_pipeline::apply_unframed(context, clip, visual, source)?
    })
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
