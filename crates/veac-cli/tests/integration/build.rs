use super::support::*;

const ENTRY: &str = r##"import "./brand.veac" as brand;
fn main(context: Context) -> Project {
  let scene_item = item(identifier("background"), item_enabled(), during(0s, 1s),
    source_generated(generator_solid(brand.background())), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(scene_item);
  let timeline = sequence(identifier("main"), "CLI build",
    sequence_settings(canvas(64px, 36px), frame_rate(24, 1), 48000))
    .with_layer(visual);
  project(identifier("cli-build"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"##;

const LOCAL_IMAGE: &str = r#"fn main(context: Context) -> Project {
  let poster = image_resource(identifier("poster"), resource_file("assets/poster.png"),
    sha256("0000000000000000000000000000000000000000000000000000000000000000"));
  let scene_item = item(identifier("poster"), item_enabled(), during(0s, 1s),
    source_media(poster), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(scene_item);
  let timeline = sequence(identifier("main"), "CLI image build",
    sequence_settings(canvas(64px, 36px), frame_rate(24, 1), 48000))
    .with_layer(visual);
  project(identifier("image-build"), project_settings(600)).with_resource(poster)
    .with_sequence(timeline).entry(timeline)
}
"#;

fn fixture() -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let entry = source_file(&temp, ENTRY);
    let module = temp.path().join("brand.veac");
    std::fs::write(
        &module,
        "module { export fn background() -> color { #2b6574ff } }\n",
    )
    .unwrap();
    (temp, entry, module)
}

fn build(entry: &std::path::Path, extra: &[&str]) -> std::process::Output {
    let mut command = veac();
    command.arg("build").arg(entry).args(extra);
    command.output().unwrap()
}

#[test]
fn build_stdout_is_canonical_revisioned_and_deterministic() {
    let (_temp, entry, _module) = fixture();
    let first = build(&entry, &["--revision", "7"]);
    let second = build(&entry, &["--revision", "7"]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, second.stdout);
    let envelope =
        veac_ir::decode_canonical_json(std::str::from_utf8(&first.stdout).unwrap()).unwrap();
    assert!(envelope.project.id.as_str().starts_with("prj_"));
    assert_eq!(envelope.project.revision, 7);
    assert!(veac_ir::validate(&envelope).is_ok());
}

#[test]
fn build_emit_ir_writes_once_and_protects_the_source_graph() {
    let (temp, entry, module) = fixture();
    let output = temp.path().join("project.json");
    let result = build(&entry, &["--emit-ir", output.to_str().unwrap()]);
    assert!(result.status.success());
    assert!(veac_ir::decode_canonical_json(&std::fs::read_to_string(output).unwrap()).is_ok());

    for protected in [&entry, &module] {
        let original = std::fs::read(protected).unwrap();
        let result = build(&entry, &["--emit-ir", protected.to_str().unwrap()]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("OUTPUT_OVERWRITES_INPUT"));
        assert_eq!(std::fs::read(protected).unwrap(), original);
    }
}

#[test]
fn build_json_diagnostics_preserve_program_codes_and_locations() {
    let temp = tempdir().unwrap();
    let missing = source_file(&temp, "fn helper(value: int) -> int { value }\n");
    let output = veac()
        .args([
            "--diagnostic-format",
            "json",
            "build",
            missing.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(
        json["diagnostics"][0]["code"],
        "PROGRAM_EXECUTABLE_MAIN_MISSING"
    );
    assert_eq!(
        json["diagnostics"][0]["source_span"]["path"],
        missing.display().to_string()
    );

    let runtime_source = format!(
        "fn fail(value: int) -> int {{ let ignored = 1 / value; 10 }}\n{}",
        EXECUTABLE_SOURCE.replacen("frame_rate(10, 1)", "frame_rate(fail(0), 1)", 1)
    );
    let runtime = source_file(&temp, &runtime_source);
    let output = build(&runtime, &[]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("PROGRAM_EXECUTABLE_RUNTIME"));
}

#[test]
fn build_emits_canonical_local_image_material_and_reference() {
    let temp = tempdir().unwrap();
    let entry = source_file(&temp, LOCAL_IMAGE);
    let output = build(&entry, &[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope =
        veac_ir::decode_canonical_json(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    let material = &envelope.project.materials[0];
    assert_eq!(
        material.source,
        veac_ir::MaterialSource::File {
            uri: "assets/poster.png".to_owned()
        }
    );
    assert_eq!(
        material.identity.as_ref().unwrap().digest,
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
    let clip = &envelope.project.sequences[0].tracks[0].clips[0];
    assert!(matches!(
        &clip.source,
        veac_ir::ClipSource::Media { material_id } if material_id == &material.id
    ));
    assert!(clip.source_mapping.is_some());
}

#[test]
fn build_keeps_local_material_uris_anchored_to_the_source_root() {
    let temp = tempdir().unwrap();
    let entry = source_file(&temp, LOCAL_IMAGE);
    let rooted = temp.path().join("project.json");
    assert!(build(&entry, &["--emit-ir", rooted.to_str().unwrap()])
        .status
        .success());

    let other = temp.path().join("other");
    std::fs::create_dir(&other).unwrap();
    let displaced = other.join("project.json");
    let output = build(&entry, &["--emit-ir", displaced.to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("OUTPUT_MATERIAL_BASE_MISMATCH"));
    assert!(!displaced.exists());
}
