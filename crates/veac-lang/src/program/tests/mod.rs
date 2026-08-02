mod component_composition;
mod components;
mod coverage_errors;
mod coverage_modules;
mod definition_closure;
mod definition_validation;
mod hygiene;
mod loader;
mod names;
mod parser;
mod presets;
mod rendering;
mod resolution;
mod slot_contract;
mod text;

const SETTINGS: &str = r#"settings {
    timebase 1/1000;
    canvas 640px by 360px;
    frame-rate 30fps;
    sample-rate 48000hz;
  }"#;

fn entry(declarations: &str, body: &str) -> String {
    format!("{declarations}\nproject test {{\n  {SETTINGS}\n  entry sequence main;\n  {body}\n}}")
}

fn empty_entry(declarations: &str) -> String {
    entry(declarations, "sequence main {}")
}

fn error_code(source: &str) -> &'static str {
    super::compile_source(source).unwrap_err().as_slice()[0].code
}
