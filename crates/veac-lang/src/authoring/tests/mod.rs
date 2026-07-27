mod annotations;
mod apply;
mod apply_rejections;
mod audio;
mod captions;
mod color;
mod core;
mod coverage_annotation_errors;
mod coverage_annotation_payload;
mod coverage_annotation_support;
mod coverage_annotation_targets;
mod coverage_ast;
mod coverage_color_enums;
mod coverage_effect_format;
mod coverage_interpolation;
mod coverage_lexer;
mod coverage_lower_settings;
mod coverage_lower_values;
mod coverage_mapping;
mod coverage_mapping_lower;
mod coverage_modifier_effect;
mod coverage_output_aux;
mod coverage_output_color;
mod coverage_output_rate;
mod coverage_output_support;
mod coverage_output_video;
mod coverage_resources_lower;
mod coverage_template_slots;
mod coverage_text_animation;
mod coverage_text_enums;
mod diagnostics;
mod example_values;
mod examples;
mod format;
mod generators;
mod lowering;
mod mapping;
mod masks;
mod modifiers;
mod multicam;
mod output_validation;
mod outputs;
mod relations_typed;
mod sources;
mod structures;
mod template_slots;
mod text;

pub(super) const SETTINGS: &str = r#"
  settings {
    timebase 1/1000000;
    canvas 1920px by 1080px;
    frame-rate 30000/1001fps;
    sample-rate 48000hz;
  }
"#;

pub(super) fn project(body: &str) -> String {
    format!("project demo {{\n{SETTINGS}\n  entry sequence main;\n{body}\n}}")
}
