use std::fs;

use tempfile::tempdir;
use veac_lang::program::expression::FunctionId;
use veac_lang::program::{
    build_path, build_source, prepare_source, BuiltProgram, Diagnostics, ExecutableBuild,
};

fn project(declarations: &str, duration: &str) -> String {
    format!(
        r#"{declarations}
fn main(context: Context) -> Project {{
  let result = item(identifier("result"), item_enabled(), during(0s, {duration}),
    source_generated(generator_solid(#00000000)), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(result);
  let timeline = sequence(identifier("main"), "名义类型方法",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(visual);
  project(identifier("nominal-methods"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}}"#
    )
}

fn duration(program: &BuiltProgram) -> veac_ir::RationalTime {
    program.envelope().project.sequences[0].tracks[0].clips[0]
        .record_range
        .duration
}

#[test]
fn function_method_and_private_method_calls_share_one_resolved_graph() {
    let declarations = r#"struct Timing { duration: time, }
impl Timing @timing {
  fn padded(self, extra: time) -> time { self.duration + extra }
  fn finish(self) -> time { self.padded(private_padding()) }
}
fn private_padding() -> time { 1s }
fn render(value: Timing) -> time { value.finish() }
"#;
    let compiled =
        build_source(&project(declarations, "render(Timing { duration: 2s, })")).unwrap();
    assert_eq!(duration(&compiled).value, 1_800);
    validate(&compiled);
}

#[test]
fn methods_fill_positional_and_named_default_parameters() {
    let declarations = r#"struct Timing { duration: time, }
impl Timing @timing {
  fn padded(self, extra: time = 500ms, factor: scalar = 2.0) -> time {
    (self.duration + extra) * factor
  }
}"#;
    for (expression, expected) in [
        ("Timing { duration: 1s, }.padded()", 1_800),
        ("Timing { duration: 1s, }.padded(1s)", 2_400),
        ("Timing { duration: 1s, }.padded(factor: 3.0)", 2_700),
    ] {
        let compiled = build_source(&project(declarations, expression)).unwrap();
        assert_eq!(duration(&compiled).value, expected);
    }
}

#[test]
fn imported_exported_method_retains_private_method_and_helper() {
    let module = r#"module {
  export struct Timing { duration: time, }
  impl Timing @timing {
    export fn finish(self) -> time { self.padded() }
    fn padded(self) -> time { helper(self.duration) }
  }
  fn helper(value: time) -> time { value + 1s }
}"#;
    let declarations = r#"import "./timing.veac" as timing;
fn render(value: timing.Timing) -> time { value.finish() }"#;
    let compiled = compile_module(module, declarations).unwrap();
    assert_eq!(duration(&compiled).value, 1_800);
    validate(&compiled);

    let private_call = declarations.replace("value.finish()", "value.padded()");
    let error = compile_module(module, &private_call).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_FUNCTION_EXPRESSION");
    assert!(error.as_slice()[0]
        .message
        .contains("EXPRESSION_UNKNOWN_METHOD"));
}

#[test]
fn resolved_method_cycles_are_rejected_even_when_unused() {
    let declarations = r#"struct Timing {}
impl Timing @timing {
  fn first(self) -> time { self.second() }
  fn second(self) -> time { self.first() }
}"#;
    let error = build_source(&project(declarations, "1s")).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_FUNCTION_CYCLE");
}

#[test]
fn foreign_type_extensions_are_rejected() {
    let module = "module { export struct Timing { duration: time, } }";
    let declarations = r#"import "./timing.veac" as timing;
impl timing.Timing @timing { fn finish(self) -> time { self.duration } }"#;
    let error = compile_module(module, declarations).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_FOREIGN_IMPL");
}

#[test]
fn multiple_named_impl_blocks_execute_for_one_receiver() {
    let declarations = r#"struct Timing { duration: time, }
impl Timing @base { fn base(self) -> time { self.duration } }
impl Timing @padding { fn padded(self) -> time { self.base() + 1s } }
fn render(value: Timing) -> time { value.padded() }"#;
    let compiled =
        build_source(&project(declarations, "render(Timing { duration: 2s, })")).unwrap();
    assert_eq!(duration(&compiled).value, 1_800);
}

#[test]
fn implementation_identity_is_unique_per_receiver() {
    let duplicate = r#"struct Timing {}
impl Timing @shared {}
impl Timing @shared {}"#;
    let error = build_source(&project(duplicate, "1s")).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_DUPLICATE_IMPL_ID");

    let distinct = r#"struct First {}
struct Second {}
impl First @shared {}
impl Second @shared {}"#;
    assert!(build_source(&project(distinct, "1s")).is_ok());
}

#[test]
fn moving_a_method_between_impl_identities_preserves_its_function_id() {
    let declarations = r#"struct Timing { duration: time, }
impl Timing @base { fn finish(self) -> time { self.duration } }"#;
    let source = project(declarations, "Timing { duration: 2s, }.finish()");
    let first = prepare_source(&source).unwrap();
    let moved = prepare_source(&source.replace("@base", "@presentation")).unwrap();
    assert_eq!(
        method_id(&first, "Timing", "finish"),
        method_id(&moved, "Timing", "finish")
    );
}

fn method_id(build: &ExecutableBuild, receiver: &str, method: &str) -> FunctionId {
    let receiver = build.type_registry().resolve(receiver).unwrap().id();
    build
        .method_registry()
        .lookup(receiver, method)
        .unwrap()
        .signature()
        .function_id()
}

fn compile_module(module: &str, declarations: &str) -> Result<BuiltProgram, Diagnostics> {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("timing.veac"), module).unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(
        &entry,
        project(declarations, "render(timing.Timing { duration: 2s, })"),
    )
    .unwrap();
    build_path(&entry)
}

fn validate(program: &BuiltProgram) {
    veac_ir::validate(program.envelope()).unwrap();
}
