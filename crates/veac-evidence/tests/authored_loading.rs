use std::collections::BTreeMap;
use std::path::Path;

use veac_evidence::*;
use veac_lang::program::{LoadedSource, SourceLoader};

const ALL_FEATURES: &str = include_str!("fixtures/authored_all_features.veac");
const HELPER: &str = r#"module {
  import "veac/evidence.veac" as evidence_api;
  export fn schema_version() -> int { 1 }
}"#;

fn imported_source() -> String {
    format!(
        "import \"./helper.veac\" as helper;\n{}",
        ALL_FEATURES.replace(
            "schema_version: 1",
            "schema_version: helper.schema_version()"
        )
    )
}

#[test]
fn delegated_loader_preserves_authored_graph_and_owns_the_builtin_module() {
    let loader = MapLoader(BTreeMap::from([
        ("helper.veac".to_owned(), HELPER.to_owned()),
        (
            EVIDENCE_MODULE_ID.to_owned(),
            "module { export struct EvidenceSuite { broken: int, } }".to_owned(),
        ),
    ]));
    let entry = LoadedSource {
        id: "evidence.veac".to_owned(),
        source: imported_source(),
    };
    let authored = build_evidence_with_loader(entry, &loader).unwrap();
    assert_eq!(authored.sources.len(), 2);
    assert!(authored.sources.contains_key("evidence.veac"));
    assert!(authored.sources.contains_key("helper.veac"));
    assert!(!authored.sources.contains_key(EVIDENCE_MODULE_ID));
}

#[test]
fn path_and_explicit_root_apis_keep_stable_source_ids() {
    let temp = tempfile::tempdir().unwrap();
    let contracts = temp.path().join("contracts");
    std::fs::create_dir(&contracts).unwrap();
    let entry = contracts.join("evidence.veac");
    std::fs::write(&entry, imported_source()).unwrap();
    std::fs::write(contracts.join("helper.veac"), HELPER).unwrap();

    let rooted = build_evidence_root_path(temp.path(), Path::new("contracts/evidence.veac"))
        .expect("explicit project root builds nested evidence");
    assert_eq!(rooted.root_module, "contracts/evidence.veac");
    assert!(rooted.sources.contains_key("contracts/helper.veac"));

    let (root, local) = build_evidence_path(&entry).expect("entry-relative evidence builds");
    assert_eq!(root, std::fs::canonicalize(&contracts).unwrap());
    assert_eq!(local.root_module, "evidence.veac");
    assert!(local.sources.contains_key("helper.veac"));
    assert_eq!(rooted.suite_sha256, local.suite_sha256);
}

#[test]
fn explicit_root_rejects_entry_escape_and_missing_files() {
    let temp = tempfile::tempdir().unwrap();
    let escape = build_evidence_root_path(temp.path(), Path::new("../outside.veac"));
    let EvidenceAuthoringError::Load(message) = escape.unwrap_err() else {
        panic!("root escape must fail during source loading");
    };
    assert!(message.contains("root-confined"));

    let missing = build_evidence_path(&temp.path().join("missing/evidence.veac"));
    assert!(matches!(missing, Err(EvidenceAuthoringError::Load(_))));
}

struct MapLoader(BTreeMap<String, String>);

impl SourceLoader for MapLoader {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let id = if requested == "./helper.veac" {
            "helper.veac"
        } else {
            requested
        };
        self.0
            .get(id)
            .map(|source| LoadedSource {
                id: id.to_owned(),
                source: source.clone(),
            })
            .ok_or_else(|| format!("{importer} cannot resolve {requested}"))
    }
}
