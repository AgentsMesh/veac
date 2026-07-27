#[path = "../../../tests/support/mod.rs"]
mod support;

mod arithmetic_defenses;
mod caption_speaker;
mod color;
mod color_matrix;
mod contract;
mod defenses;
mod duration;
mod mechanism_helpers;
mod mechanisms;
mod multicam;
mod output;
mod output_selection;
mod source_time;
mod text_geometry;
mod text_graphics;

#[path = "../../../tests/apply_edge_resolution.rs"]
mod integration_apply_edge_resolution;
#[path = "../../../tests/audio_color_resolution.rs"]
mod integration_audio_color_resolution;
#[path = "../../../tests/composition_resolution.rs"]
mod integration_composition_resolution;
#[path = "../../../tests/empty_sequence_resolution.rs"]
mod integration_empty_sequence_resolution;
#[path = "../../../tests/mechanism_resolution.rs"]
mod integration_mechanism_resolution;
#[path = "../../../tests/multicam_resolution.rs"]
mod integration_multicam_resolution;
#[path = "../../../tests/relation_projection.rs"]
mod integration_relation_projection;
#[path = "../../../tests/resolution_errors.rs"]
mod integration_resolution_errors;
#[path = "../../../tests/resolution_success.rs"]
mod integration_resolution_success;
#[path = "../../../tests/serialization_contract.rs"]
mod integration_serialization_contract;
#[path = "../../../tests/source_time_resolution.rs"]
mod integration_source_time_resolution;
