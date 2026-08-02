mod animation;
mod annotation;
mod apply;
mod audio;
mod audio_graph;
mod budget;
mod clip;
mod color;
mod composition;
mod diagnostics;
mod effects;
mod generator;
mod graph;
mod mask;
mod material;
mod media_type;
mod multicam;
mod output;
mod output_compat;
mod probe;
mod project;
mod relation;
mod sequence;
mod source_time;
mod template;
mod text;
mod values;
mod visual;

use std::collections::{BTreeMap, BTreeSet};

pub use diagnostics::{Diagnostic, ValidationErrors};
pub use material::project_relative_uri_valid;
pub use output_compat::{
    audio_container_compatible, audio_output_valid, ffmpeg_dimensions_valid,
    ffmpeg_sample_rate_valid, image_sequence_range_valid, image_sequence_start_valid,
    input_audio_stream_valid, input_video_geometry_valid, mxf_geometry_valid,
    normalized_video_dimensions, output_file_compatible, pixel_geometry_valid,
    render_geometry_valid, video_color_delivery_valid, video_container_compatible,
    video_delivery_valid, video_settings_valid, FFMPEG_INT_MAX, FLAC_MAX_SAMPLE_RATE,
    MAX_CHANNEL_LAYOUT_BYTES, MAX_DIMENSION, MAX_FRAME_PIXELS, MAX_FRAME_RATE,
    MAX_INPUT_AUDIO_CHANNELS, MAX_INPUT_AUDIO_SAMPLE_RATE, MAX_VIDEO_BITRATE, MAX_VIDEO_BUFFER,
    PCM_MAX_SAMPLE_RATE,
};
pub use probe::container_format_valid;

use crate::*;

pub fn validate(envelope: &ProjectEnvelope) -> Result<(), ValidationErrors> {
    let mut validator = Validator::default();
    validator.envelope(envelope);
    if validator.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(ValidationErrors::new(validator.diagnostics))
    }
}

pub fn validate_sequence_relations(
    sequence: &Sequence,
    relations: &[Relation],
) -> Result<(), ValidationErrors> {
    let graph = RelationGraph::scoped(sequence, relations);
    let mut validator = Validator::default();
    validator.relation_scope(std::slice::from_ref(sequence), relations, &graph);
    if validator.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(ValidationErrors::new(validator.diagnostics))
    }
}

#[derive(Default)]
struct Validator {
    diagnostics: Vec<Diagnostic>,
    material_ids: BTreeMap<String, MaterialKind>,
    sequence_ids: BTreeSet<String>,
    sequence_audio: BTreeMap<String, bool>,
    track_ids: BTreeSet<String>,
    item_ids: BTreeSet<String>,
    effect_ids: BTreeSet<String>,
    keyframe_ids: BTreeSet<String>,
    relation_ids: BTreeSet<String>,
    apply_ids: BTreeSet<String>,
    apply_stage_ids: BTreeSet<String>,
    relation_group_members: BTreeSet<String>,
    relation_av_link_members: BTreeSet<String>,
    multicam_groups: BTreeMap<String, BTreeSet<String>>,
    output_ids: BTreeSet<String>,
    deliverable_ids: BTreeSet<String>,
}

pub(crate) use values::json_value_is_ijson;
