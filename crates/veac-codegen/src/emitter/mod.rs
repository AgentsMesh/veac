mod alpha_merge;
pub(crate) mod animation;
mod apply;
mod apply_slice;
mod apply_stack;
pub(crate) mod audio;
mod audio_fades;
mod audio_filters;
mod audio_processing;
mod audio_sidechain;
mod audio_source;
mod audio_time_map;
mod audio_transition_fades;
mod blend;
mod canvas;
mod color;
mod color_lut;
mod color_output;
mod color_space;
mod color_stage;
mod command;
mod context_impl;
mod delivery;
mod delivery_animated;
mod delivery_audio;
mod delivery_audio_file;
mod delivery_caption;
mod delivery_hls;
mod delivery_image;
mod delivery_scope;
mod delivery_still;
mod effect_keying;
mod effect_video;
mod effects;
mod error;
mod filter_resource;
mod frame_hold;
mod generated;
mod generated_geometry;
mod generated_gradient;
mod generated_shape;
pub(crate) mod geometry;
pub(crate) mod graph;
mod hls_encoding;
mod input;
mod layer;
mod layer_place;
mod layer_placement;
mod mask;
mod mask_expression;
mod mask_path;
mod matte;
mod multicam_source;
mod output;
mod output_video;
mod preflight;
mod preparation;
pub(crate) mod process_owner;
mod render_segment;
mod resource;
mod rgb_planes;
mod sequence;
mod shadow;
mod source;
mod source_boundary;
mod temporal;
mod text;
pub(crate) mod time;
mod transition;
mod video_source;
mod video_time_map;
mod visual;
mod visual_crop;
mod visual_extent;
mod visual_frame;
mod visual_pipeline;
mod visual_pivot;
mod visual_shear;

#[cfg(test)]
#[path = "../unit_tests/emitter_internal_tests.rs"]
mod internal_tests;

#[cfg(test)]
#[path = "../unit_tests/audio_source_internal_tests.rs"]
mod audio_source_internal_tests;

#[cfg(test)]
#[path = "../unit_tests/transition_internal_tests.rs"]
mod transition_internal_tests;

#[cfg(test)]
#[path = "../unit_tests/rgb_planes_tests.rs"]
mod rgb_planes_tests;

pub use command::*;
pub use delivery::emit_all;
pub use error::{CodegenDiagnostic, CodegenErrorKind, CodegenErrors};

use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{AlphaMode, Deliverable, VideoDeliverable};
use veac_plan::ResolvedRenderPlan;

use canvas::Canvas;
use graph::Graph;

/// Leaves headroom below Linux's per-argument `MAX_ARG_STRLEN` boundary.
pub const MAX_INLINE_FILTER_GRAPH_BYTES: usize = 120_000;
const FILTER_GRAPH_ENVELOPE_RESERVE_BYTES: usize = 32 * 1024;
const FILTER_GRAPH_ASS_RESERVE_BYTES: usize = 16 * 1024;
/// Retains room for the rest of an artifact JSON envelope around the graph.
pub const MAX_FILTER_GRAPH_BYTES: usize =
    veac_artifact::MAX_ARTIFACT_JSON_STRING_BYTES - FILTER_GRAPH_ENVELOPE_RESERVE_BYTES;
/// Raw ASS is Base64 encoded into the graph; reserve space for filter syntax and other nodes.
pub const MAX_ASS_PAYLOAD_BYTES: usize =
    (MAX_FILTER_GRAPH_BYTES - FILTER_GRAPH_ASS_RESERVE_BYTES) / 4 * 3;

fn emit_video_delivery(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    deliverable: &Deliverable,
    video: &VideoDeliverable,
) -> Result<BackendCommand, CodegenErrors> {
    let context = EmitContext::new_visual(plan, bindings, deliverable, video.video.alpha)?;
    context.build_video(video)
}

struct EmitContext<'a> {
    plan: &'a ResolvedRenderPlan,
    bindings: &'a ExecutionBindings,
    deliverable: &'a Deliverable,
    alpha: AlphaMode,
    input_routes: input::InputRoutes,
    canvas: Canvas,
    graph: Graph,
    filter_bindings: Vec<BackendFilterBinding>,
    preparations: Vec<BackendPreparation>,
}
