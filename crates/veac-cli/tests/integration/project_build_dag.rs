use super::project_build::{target_with_picture, TARGET};
use super::support::*;

const PROJECT: &str = r#"fn workspace() -> ProjectManifest {
  ProjectManifest {
    schema: "veac.project",
    version: 1,
    id: identifier("dag"),
    paths: ProjectPaths {
      source_base: "sources",
      material_root: "materials",
      build_root: "build",
      cache_root: ".cache/veac",
      delivery_root: "dist",
    },
    defaults: ProjectDefaults {
      profile: OptionalIdentifier.None,
      locale: OptionalIdentifier.None,
      max_instances_per_target: 4,
      max_total_instances: 8,
    },
    locales: [],
    profiles: [],
    targets: [
      ProjectTarget {
        id: identifier("producer"),
        entry: ProjectTargetEntry.Veac { source: "producer.veac", },
        localized: false,
        inputs: [],
        profiles: [],
        axes: [],
        needs: [],
        outputs: [
          ProjectOutput.Media { id: identifier("video"), media_type: MediaType.Video, }
        ],
        deliveries: [],
      },
      ProjectTarget {
        id: identifier("consumer"),
        entry: ProjectTargetEntry.Veac { source: "consumer.veac", },
        localized: false,
        inputs: [
          ProjectInput {
            id: identifier("picture"),
            source: ProjectInputSource.Artifact {
              target: TargetRef {
                target: identifier("producer"),
                profile: OptionalIdentifier.None,
                selector: InstanceSelector.Same,
              },
              output: identifier("video"),
            },
          }
        ],
        profiles: [],
        axes: [],
        needs: [],
        outputs: [
          ProjectOutput.Media { id: identifier("final"), media_type: MediaType.Video, }
        ],
        deliveries: [
          ProjectDelivery.File {
            id: identifier("final-file"),
            output: identifier("final"),
            destination: "final.mp4",
          }
        ],
      }
    ],
  }
}
"#;

#[test]
fn project_build_fans_verified_outputs_into_downstream_material_inputs() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    let receipt = temp.path().join("build/receipt.json");
    let sources = temp.path().join("sources");
    std::fs::create_dir(&sources).unwrap();
    std::fs::write(&entry, PROJECT).unwrap();
    std::fs::write(sources.join("producer.veac"), without_parameters(TARGET)).unwrap();
    std::fs::write(sources.join("consumer.veac"), consumer()).unwrap();
    std::fs::create_dir(temp.path().join("materials")).unwrap();

    let output = veac()
        .args(["project", "build"])
        .arg(&entry)
        .arg("--receipt")
        .arg(&receipt)
        .output()
        .unwrap();
    let receipt_bytes = std::fs::read(&receipt).unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&receipt_bytes)
    );
    let value: serde_json::Value = serde_json::from_slice(&receipt_bytes).unwrap();
    assert_eq!(value["outcome"], "succeeded");
    assert_eq!(value["nodes"].as_array().unwrap().len(), 2);
    assert!(value["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .all(|node| node["status"] == "executed"));
    assert!(temp.path().join("dist/final.mp4").is_file());
}

fn without_parameters(source: &str) -> String {
    source
        .replace("input parameter profile: text;\n", "")
        .replace("input parameter locale: text;\n", "")
        .replace("input parameter theme: text;\n", "")
}

fn consumer() -> String {
    without_parameters(&target_with_picture())
        .replace("identifier(\"video\")", "identifier(\"final\")")
        .replace(
            "image_resource(identifier(\"picture\"), resource_file(picture.path), sha256(picture.sha256))",
            "video_resource(identifier(\"picture\"), resource_file(picture.path), sha256(picture.sha256), stream_intent(stream_auto(), stream_disabled()))",
        )
}
