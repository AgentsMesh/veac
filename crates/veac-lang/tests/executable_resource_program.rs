use std::fs;

use tempfile::tempdir;
use veac_lang::program::{build_path, DomainOperationId as Op};

const ENTRY: &str = r#"import "./images.veac" as images;
fn main(context: Context) -> Project {
  let labels = map(["海报", "封面"], fn(value: text) -> text effect pure { value });
  let retained_labels = labels;
  let poster = images.local_image(identifier("poster"));
  let cover = images.local_image(identifier("cover"));
  let timeline = sequence(
    identifier("main"), "资源画廊",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
  );
  project(identifier("gallery"), project_settings(600))
    .with_resource(poster).with_resource(cover)
    .with_sequence(timeline).entry(timeline)
}
"#;

const MODULE: &str = r#"module {
  export fn local_image(key: identifier) -> Resource {
    image_resource(key, resource_file("assets/poster.png"),
      sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"))
  }
}
"#;

#[test]
fn module_helper_and_pure_map_emit_static_project_resources() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, ENTRY).unwrap();
    fs::write(temp.path().join("images.veac"), MODULE).unwrap();

    let built = build_path(&entry).unwrap();
    let project = &built.envelope().project;
    assert_eq!(project.materials.len(), 2);
    assert!(project
        .materials
        .iter()
        .all(|material| material.id.as_str().starts_with("med_")));
    assert!(project.materials.iter().all(|material| {
        material
            .identity
            .as_ref()
            .is_some_and(|identity| identity.digest == "a".repeat(64))
    }));
    let mut paths = project
        .materials
        .iter()
        .map(|material| material.authorship.as_ref().unwrap().logical_path.clone())
        .collect::<Vec<_>>();
    paths.sort_by_key(|path| path.last().unwrap().as_str().to_owned());
    assert_eq!(
        paths[0]
            .iter()
            .map(|part| part.as_str())
            .collect::<Vec<_>>(),
        vec!["gallery", "resource", "cover"]
    );
    assert_eq!(
        paths[1]
            .iter()
            .map(|part| part.as_str())
            .collect::<Vec<_>>(),
        vec!["gallery", "resource", "poster"]
    );
    for material in &project.materials {
        let constructor = &material.authorship.as_ref().unwrap().events[0];
        assert_eq!(constructor.operation.0, Op::ImageResource.opcode());
        assert_eq!(constructor.origin.source.as_str(), "images.veac");
        assert_eq!(constructor.definition.name.as_str(), "local_image");
        assert_eq!(constructor.call_stack[0].function.as_str(), "main");
        assert_eq!(constructor.call_stack.len(), 1);
    }
    let project_events = &project.authorship.as_ref().unwrap().entity.events;
    assert!(
        project_events
            .iter()
            .filter(|event| event.operation.0 == Op::ProjectWithResource.opcode())
            .count()
            == 2
    );
    assert!(veac_ir::validate(built.envelope()).is_ok());
}
