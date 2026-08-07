use std::fs;
use std::process::Command;

use tempfile::tempdir;
use veac_lang::program::{build_path, DomainOperationId as Op};
use veac_lang::source_edit::{source_graph_revision, SourceModule};

const ENTRY: &str = r#"import "./library.veac" as library;
fn main(context: Context) -> Project {
  let labels = map(["一", "二"], fn(value: text) -> text effect pure { value });
  let retained_labels = labels;
  let one = library.card(identifier("one"));
  let two = library.card(identifier("two"));
  let state = track_state(
    track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked()
  );
  let visual = visual_layer(
    identifier("visual"), 0, placement_free(), state, track_routing_default()
  ).with_item(one).with_item(two);
  let timeline = sequence(
    identifier("main"), "来源追踪",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
  ).with_layer(visual);
  project(identifier("project"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

const MODULE: &str = r#"module {
  export fn card(key: identifier) -> Item {
    item(key, item_enabled(), during(0s, 2s),
      source_generated(generator_solid(#204060ff)), source_timing_native())
      .with_effect(video_blur_effect(
        identifier("blur"), effect_enabled(effect_window_full()), length_constant(2px)
      ))
  }
}
"#;

#[test]
fn executable_authorship_tracks_modules_pure_maps_updates_and_identity() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, ENTRY).unwrap();
    fs::write(temp.path().join("library.veac"), MODULE).unwrap();

    let first = build_path(&entry).unwrap();
    let second = build_path(&entry).unwrap();
    assert_eq!(
        veac_ir::canonical_json(first.envelope()).unwrap(),
        veac_ir::canonical_json(second.envelope()).unwrap()
    );
    let project = &first.envelope().project;
    assert_program_identity(first.envelope());
    let authorship = project.authorship.as_ref().unwrap();
    assert_path(&authorship.entity, &["project"]);

    let sequence = &project.sequences[0];
    let veac_ir::SequenceAuthorship::Veac { entity, tracks, .. } =
        sequence.authorship.as_ref().unwrap()
    else {
        panic!("expected VEAC authorship")
    };
    assert_path(entity, &["project", "sequence", "main"]);
    assert_eq!(tracks.len(), 1);
    let layer = &tracks[0].entity;
    assert_path(layer, &["project", "sequence", "main", "layer", "visual"]);
    assert_eq!(layer.events[1].kind, veac_ir::AuthorshipEventKind::Update);
    assert_eq!(layer.events[1].operation.0, Op::LayerWithItem.opcode());
    assert_eq!(layer.events[2].operation.0, Op::LayerWithItem.opcode());

    let clips = &sequence.tracks[0].clips;
    assert_eq!(clips.len(), 2);
    assert_clip(&clips[0], "one");
    assert_clip(&clips[1], "two");
    assert!(
        veac_ir::decode_canonical_json(&veac_ir::canonical_json(first.envelope()).unwrap()).is_ok()
    );
}

#[test]
fn preview_lookup_helpers_consume_real_v9_authorship() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let built = build_path(&root.join("examples/minimal/main.veac")).unwrap();
    let canonical = veac_ir::canonical_json(built.envelope()).unwrap();
    let decoded = veac_ir::decode_canonical_json(&canonical).unwrap();
    assert_eq!(decoded.schema_version, 9);

    let temp = tempdir().unwrap();
    let file = temp.path().join("project.veac.json");
    fs::write(&file, canonical).unwrap();
    let sequence = decoded.project.sequences[0].id.as_str();
    let label = decoded.project.sequences[0].tracks[0]
        .clips
        .iter()
        .find(|clip| {
            clip.authorship
                .as_ref()
                .is_some_and(|value| value.logical_path.last().unwrap().as_str() == "label")
        })
        .unwrap()
        .id
        .as_str();
    let delivery = decoded.project.authorship.as_ref().unwrap().deliveries[0]
        .render_config_id
        .as_str();
    let script = root.join("scripts/example-render-evidence/common.sh");
    let output = Command::new("bash")
        .args([
            "-c",
            "source \"$1\"; canonical_sequence_id \"$2\" main; \
             canonical_clip_id \"$2\" label; delivery_config_id \"$2\" preview",
            "v9-authorship-lookup",
        ])
        .arg(script)
        .arg(file)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{sequence}\n{label}\n{delivery}\n")
    );
}

fn assert_program_identity(value: &veac_ir::ProjectEnvelope) {
    let executable = &value.executable;
    assert_eq!(executable.core_version, veac_ir::CURRENT_CORE_VERSION);
    assert_eq!(
        executable.domain_opset_version,
        veac_ir::CURRENT_DOMAIN_OPSET_VERSION
    );
    assert_eq!(executable.digests.domain_registry_sha256.len(), 64);
    assert_eq!(executable.digests.main_core_sha256.len(), 64);
    assert_eq!(executable.digests.source_graph_sha256.len(), 64);
    let expected = source_graph_revision(&[
        SourceModule::utf8("library.veac", MODULE),
        SourceModule::utf8("main.veac", ENTRY),
    ])
    .unwrap();
    assert_eq!(
        executable.digests.source_graph_sha256,
        expected.source_graph_sha256
    );
}

fn assert_clip(clip: &veac_ir::Clip, key: &str) {
    let value = clip.authorship.as_ref().unwrap();
    assert_path(
        value,
        &[
            "project", "sequence", "main", "layer", "visual", "item", key,
        ],
    );
    let constructor = &value.events[0];
    assert_eq!(constructor.kind, veac_ir::AuthorshipEventKind::Constructor);
    assert_eq!(constructor.operation.0, Op::Item.opcode());
    assert_eq!(constructor.origin.source.as_str(), "library.veac");
    assert_eq!(constructor.definition.name.as_str(), "card");
    assert_eq!(
        constructor.definition.kind,
        veac_ir::AuthoredDefinitionKind::Function
    );
    let stack = &constructor.call_stack;
    assert_eq!(stack.len(), 1);
    assert_eq!(stack[0].function.as_str(), "main");
    let update = &value.events[1];
    assert_eq!(update.kind, veac_ir::AuthorshipEventKind::Update);
    assert_eq!(update.operation.0, Op::ItemWithEffect.opcode());
    assert_eq!(update.origin.source.as_str(), "library.veac");
    assert_eq!(update.definition.name.as_str(), "card");
}

fn assert_path(value: &veac_ir::EntityAuthorship, expected: &[&str]) {
    assert_eq!(
        value
            .logical_path
            .iter()
            .map(veac_ir::LogicalPathSegment::as_str)
            .collect::<Vec<_>>(),
        expected.to_vec()
    );
}
