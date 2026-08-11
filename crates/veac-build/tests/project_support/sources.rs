pub const PROJECT: &str = r#"import "./plate-module.veac" as plate;
fn main(context: Context) -> Project {
  let timeline = sequence(identifier("main"), plate.title(),
    sequence_settings(canvas(16px, 16px), frame_rate(1, 1), 8000));
  project(identifier("fixture"), project_settings(60))
    .with_sequence(timeline).entry(timeline)
}
"#;

pub const PROJECT_MODULE: &str = "module { export fn title() -> text { \"Fixture\" } }\n";

pub const EVIDENCE: &str = r#"import "./evidence-helper.veac" as helper;
fn evidence() -> EvidenceSuite {
  EvidenceSuite {
    schema_version: 1,
    id: identifier(helper.suite_name()),
    sources: [
      EvidenceSource {
        id: identifier("rendered"),
        binding: EvidenceSourceBinding.BoundInput { input_id: identifier("source"), },
      }
    ],
    samples: [],
    regions: [],
    assertions: [
      EvidenceAssertion.DecodeComplete {
        id: identifier("decode"),
        source_id: identifier("rendered"),
        minimum_frames: 1,
      }
    ],
  }
}
"#;

pub const EVIDENCE_HELPER: &str = r#"module {
  export fn suite_name() -> text { "suite" }
}
"#;

pub const REVISED_EVIDENCE_HELPER: &str = r#"module {
  export fn suite_name() -> text { "suite-revised" }
}
"#;
