use std::fs;
use std::path::Path;

use tempfile::{tempdir, TempDir};
use veac_lang::program::{build_path, prepare_path};

const BRAND: &str = r#"module {
  export struct Card { title: text, }
  export enum Mood { Calm, Accent { color: color, }, }
  struct Hidden { value: text, }
}"#;

fn write(temp: &TempDir, name: &str, source: &str) {
    fs::write(temp.path().join(name), source).unwrap();
}

fn project(prefix: &str) -> String {
    format!(
        r#"{prefix}
fn main(context: Context) -> Project {{
  let background = item(identifier("background"), item_enabled(), during(0s, 1s),
    source_generated(generator_solid(#2b6574ff)), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(background);
  let timeline = sequence(identifier("main"), "名义类型闭包",
    sequence_settings(canvas(64px, 36px), frame_rate(24, 1), 48000))
    .with_layer(visual);
  project(identifier("nominal-closure"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}}
"#
    )
}

fn error(entry: &Path) -> veac_lang::program::Diagnostic {
    prepare_path(entry)
        .unwrap_err()
        .as_slice()
        .first()
        .unwrap()
        .clone()
}

#[test]
fn local_struct_and_enum_closures_capture_and_invoke() {
    let temp = tempdir().unwrap();
    let declarations = r#"struct Card { title: text, }
enum Mood { Calm, }
fn retain(card: Card) -> Card {
  let captured = card;
  (fn(input: Card) -> Card effect pure { captured })(card)
}
fn retain_mood(mood: Mood) -> Mood {
  (fn(input: Mood) -> Mood effect pure { input })(mood)
}
const Card retained = retain(Card { title: "local" });
const Mood retained_mood = retain_mood(Mood.Calm);"#;
    let entry = temp.path().join("main.veac");
    write(&temp, "main.veac", &project(declarations));
    let built = build_path(&entry).unwrap();
    assert!(veac_ir::validate(built.envelope()).is_ok());
}

#[test]
fn qualified_imported_struct_and_enum_closures_execute() {
    let temp = tempdir().unwrap();
    write(&temp, "brand.veac", BRAND);
    let declarations = r#"import "./brand.veac" as brand;
fn retain(card: brand.Card) -> brand.Card {
  let captured = card;
  (fn(input: brand.Card) -> brand.Card effect pure { captured })(card)
}
fn retain_mood(mood: brand.Mood) -> brand.Mood {
  (fn(input: brand.Mood) -> brand.Mood effect pure { input })(mood)
}
const brand.Card retained = retain(brand.Card { title: "imported" });
const brand.Mood retained_mood = retain_mood(brand.Mood.Calm);"#;
    let entry = temp.path().join("main.veac");
    write(&temp, "main.veac", &project(declarations));
    let built = build_path(&entry).unwrap();
    assert!(veac_ir::validate(built.envelope()).is_ok());
}

#[test]
fn unknown_and_private_nominal_annotations_fail_at_the_authored_type() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    write(
        &temp,
        "main.veac",
        &project("fn bad() -> int { let callback = fn(value: Missing) -> Missing effect pure { value }; 0 }"),
    );
    let missing = error(&entry);
    assert_eq!(missing.code, "PROGRAM_FUNCTION_EXPRESSION");
    assert!(missing.message.contains("PROGRAM_UNKNOWN_TYPE"));

    write(&temp, "brand.veac", BRAND);
    write(
        &temp,
        "main.veac",
        &project(
            "import \"./brand.veac\" as brand;\nfn bad() -> int { let callback = fn(value: brand.Hidden) -> brand.Hidden effect pure { value }; 0 }",
        ),
    );
    let private = error(&entry);
    assert!(private.message.contains("brand.Hidden"));
}

#[test]
fn duplicate_import_aliases_fail_before_nominal_annotation_resolution() {
    let temp = tempdir().unwrap();
    write(&temp, "first.veac", "module { export struct Card {} }");
    write(&temp, "second.veac", "module { export struct Card {} }");
    let entry = temp.path().join("main.veac");
    write(
        &temp,
        "main.veac",
        &project(
            "import \"./first.veac\" as shared;\nimport \"./second.veac\" as shared;\nfn bad() -> int { let callback = fn(value: shared.Card) -> shared.Card effect pure { value }; 0 }",
        ),
    );
    assert_eq!(error(&entry).code, "PROGRAM_IMPORT_ALIAS");
}
