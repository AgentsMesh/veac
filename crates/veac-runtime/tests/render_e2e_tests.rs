//! Real canonical-IR -> render-plan -> emitter -> FFmpeg integration tests.

#[path = "render_e2e/animation_spring.rs"]
mod animation_spring;
#[path = "render_e2e/audio.rs"]
mod audio;
#[path = "render_e2e/audio_dynamics.rs"]
mod audio_dynamics;
#[path = "render_e2e/audio_filters.rs"]
mod audio_filters;
#[path = "render_e2e/audio_sidechain.rs"]
mod audio_sidechain;
#[path = "render_e2e/audio_timeline_processing.rs"]
mod audio_timeline_processing;
#[path = "render_e2e/auto_reframe.rs"]
mod auto_reframe;
#[path = "render_e2e/bezier.rs"]
mod bezier;
#[path = "render_e2e/canvas_delivery.rs"]
mod canvas_delivery;
#[path = "render_e2e/color_curves.rs"]
mod color_curves;
#[path = "render_e2e/color_lut_resources.rs"]
mod color_lut_resources;
#[path = "render_e2e/color_matrix.rs"]
mod color_matrix;
#[path = "render_e2e/color_processing.rs"]
mod color_processing;
#[path = "render_e2e/composition.rs"]
mod composition;
#[path = "render_e2e/composition_apply_band.rs"]
mod composition_apply_band;
#[path = "render_e2e/composition_apply_items.rs"]
mod composition_apply_items;
#[path = "render_e2e/composition_apply_matte.rs"]
mod composition_apply_matte;
#[path = "render_e2e/composition_apply_stack.rs"]
mod composition_apply_stack;
#[path = "render_e2e/composition_blend_modes.rs"]
mod composition_blend_modes;
#[path = "render_e2e/composition_card.rs"]
mod composition_card;
#[path = "render_e2e/composition_circle_mask.rs"]
mod composition_circle_mask;
#[path = "render_e2e/composition_mask.rs"]
mod composition_mask;
#[path = "render_e2e/composition_mask_pixels.rs"]
mod composition_mask_pixels;
#[path = "render_e2e/composition_mask_shapes.rs"]
mod composition_mask_shapes;
#[path = "render_e2e/composition_matte.rs"]
mod composition_matte;
#[path = "render_e2e/composition_placement.rs"]
mod composition_placement;
#[path = "render_e2e/composition_placement_edges.rs"]
mod composition_placement_edges;
#[path = "render_e2e/composition_rotation_alpha.rs"]
mod composition_rotation_alpha;
#[path = "render_e2e/composition_shadow_clipping.rs"]
mod composition_shadow_clipping;
#[path = "render_e2e/composition_shadow_matte.rs"]
mod composition_shadow_matte;
#[path = "render_e2e/composition_star_mask.rs"]
mod composition_star_mask;
#[path = "render_e2e/effects.rs"]
mod effects;
#[path = "render_e2e/effects_directional_blur.rs"]
mod effects_directional_blur;
#[path = "render_e2e/example_transform_animation.rs"]
mod example_transform_animation;
#[path = "render_e2e/flip.rs"]
mod flip;
#[path = "render_e2e/frame_synthesis.rs"]
mod frame_synthesis;
#[path = "render_e2e/generated_av.rs"]
mod generated_av;
#[path = "render_e2e/generated_gradients.rs"]
mod generated_gradients;
#[path = "render_e2e/generated_shapes.rs"]
mod generated_shapes;
#[path = "render_e2e/graphics_text.rs"]
mod graphics_text;
#[path = "render_e2e/input_geometry.rs"]
mod input_geometry;
#[path = "render_e2e/keying.rs"]
mod keying;
#[path = "render_e2e/layout_matrix.rs"]
mod layout_matrix;
#[path = "render_e2e/multicam.rs"]
mod multicam;
#[path = "render_e2e/nested.rs"]
mod nested;
#[path = "render_e2e/nested_source_time.rs"]
mod nested_source_time;
#[path = "render_e2e/opaque_source_over.rs"]
mod opaque_source_over;
#[path = "render_e2e/output.rs"]
mod output;
#[path = "render_e2e/proxy_substitution.rs"]
mod proxy_substitution;
#[path = "render_e2e/render_segment.rs"]
mod render_segment;
#[path = "render_e2e/render_segment_invalid.rs"]
mod render_segment_invalid;
#[path = "render_e2e/source_boundary.rs"]
mod source_boundary;
#[path = "render_e2e/source_time.rs"]
mod source_time;
#[path = "render_e2e/source_time_policies.rs"]
mod source_time_policies;
#[path = "render_e2e/source_time_vfr.rs"]
mod source_time_vfr;
#[path = "render_e2e/stabilization.rs"]
mod stabilization;
#[path = "render_e2e/streams.rs"]
mod streams;
#[path = "render_e2e/support/mod.rs"]
mod support;
#[path = "render_e2e/temporal_bindings.rs"]
mod temporal_bindings;
#[path = "render_e2e/text.rs"]
mod text;
#[path = "render_e2e/text_advanced.rs"]
mod text_advanced;
#[path = "render_e2e/text_filter_script.rs"]
mod text_filter_script;
#[path = "render_e2e/text_geometry.rs"]
mod text_geometry;
#[path = "render_e2e/text_overflow_frame.rs"]
mod text_overflow_frame;
#[path = "render_e2e/text_shadow_pixels.rs"]
mod text_shadow_pixels;
#[path = "render_e2e/transform_shear.rs"]
mod transform_shear;
#[path = "render_e2e/transition.rs"]
mod transition;
#[path = "render_e2e/transition_frame_completion.rs"]
mod transition_frame_completion;
#[path = "render_e2e/transition_motion.rs"]
mod transition_motion;
#[path = "render_e2e/transition_typed.rs"]
mod transition_typed;
#[path = "render_e2e/transitive_inputs.rs"]
mod transitive_inputs;
