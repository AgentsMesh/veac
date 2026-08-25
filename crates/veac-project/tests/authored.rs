use std::collections::BTreeMap;

use veac_lang::program::{LoadedSource, SourceAuthority, SourceLoader};
use veac_project::*;

const MINIMAL: &str = include_str!("fixtures/authored_minimal.veac");

#[test]
fn project_source_executes_decodes_validates_and_records_provenance() {
    let project = build_project_source(MINIMAL).unwrap();
    assert_eq!(project.manifest.id, ProjectId::from("demo"));
    assert_eq!(project.root_module, "project.veac");
    assert_eq!(project.sources.len(), 1);
    assert_eq!(project.sources["project.veac"], MINIMAL);
    assert!(!project.sources.contains_key(PROJECT_MODULE_ID));
    let inventory = project
        .source_index
        .inventory(&project.source_revision())
        .unwrap();
    assert_eq!(inventory.modules.len(), 1);
    assert_eq!(inventory.modules[0].module, "project.veac");
    assert_eq!(
        project.manifest_digest,
        manifest_digest(&project.manifest).unwrap()
    );
}

#[test]
fn source_and_manifest_revisions_have_independent_identity() {
    let first = build_project_source(MINIMAL).unwrap();
    let changed = format!("{MINIMAL}\n");
    let second = build_project_source(&changed).unwrap();
    assert_ne!(first.source_revision, second.source_revision);
    assert_eq!(first.manifest_digest, second.manifest_digest);
}

#[test]
fn source_string_rejects_authored_imports_without_a_loader() {
    let source =
        "import \"./helper.veac\" as helper; fn workspace() -> ProjectManifest { helper.make() }";
    let ProjectAuthoringError::Language(error) = build_project_source(source).unwrap_err() else {
        panic!("import failure must be a language diagnostic");
    };
    assert_eq!(error.as_slice()[0].code, "PROGRAM_IMPORT_LOAD");
}

#[test]
fn delegated_loader_preserves_authored_module_graph() {
    let helper =
        "module { export fn project_id() -> identifier { identifier(\"demo\") } }".to_owned();
    let loader = MapLoader(BTreeMap::from([("helper.veac".to_owned(), helper)]));
    let entry = LoadedSource {
        id: "project.veac".to_owned(),
        source: format!(
            "import \"./helper.veac\" as helper;\n{}",
            MINIMAL.replace("identifier(\"demo\")", "helper.project_id()")
        ),
    };
    let project = build_project_with_loader(entry, &loader).unwrap();
    assert_eq!(project.sources.len(), 2);
    assert!(project.sources.contains_key("helper.veac"));
    assert!(!project.sources.contains_key(PROJECT_MODULE_ID));
}

#[test]
fn arbitrary_read_only_dependencies_only_affect_complete_identity() {
    let entry = LoadedSource {
        id: "project.veac".to_owned(),
        source: format!(
            "import \"./helper.veac\" as helper;\n{}",
            MINIMAL.replace("identifier(\"demo\")", "helper.project_id()")
        ),
    };
    let source = "module { export fn project_id() -> identifier { identifier(\"demo\") } }";
    let build = |helper: String| {
        build_project_with_loader(
            entry.clone(),
            &ReadOnlyHelperLoader(BTreeMap::from([("helper.veac".to_owned(), helper)])),
        )
        .unwrap()
    };
    let first = build(source.to_owned());
    let second = build(format!("{source}\n"));
    assert_eq!(first.sources.keys().collect::<Vec<_>>(), ["project.veac"]);
    assert_eq!(first.source_revision, second.source_revision);
    assert_ne!(
        first.complete_source_graph_revision,
        second.complete_source_graph_revision
    );
    assert_ne!(first.source_revision(), second.source_revision());
}

#[test]
fn checked_integer_decode_reports_the_logical_field_path() {
    let source = MINIMAL.replace("version: 1", "version: -1");
    let ProjectAuthoringError::Decode(error) = build_project_source(&source).unwrap_err() else {
        panic!("negative version must fail checked decoding");
    };
    assert_eq!(error.path, "manifest.version");
    assert!(error.message.contains("u32"));
}

#[test]
fn project_contract_schema_entry_points_are_stable() {
    assert_eq!(project_manifest_json_schema()["title"], "ProjectManifestV1");
    assert_eq!(
        resolved_project_graph_json_schema()["title"],
        "ResolvedTargetGraph"
    );
    assert_eq!(
        authored_project_source_index_json_schema()["title"],
        "SourceIndexInventory"
    );
}

struct MapLoader(BTreeMap<String, String>);

struct ReadOnlyHelperLoader(BTreeMap<String, String>);

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

    fn authority(&self, _source_id: &str) -> SourceAuthority {
        SourceAuthority::Project
    }
}

impl SourceLoader for ReadOnlyHelperLoader {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        MapLoader(self.0.clone()).load(importer, requested)
    }

    fn authority(&self, source_id: &str) -> SourceAuthority {
        if source_id == "helper.veac" {
            SourceAuthority::ReadOnlyDependency
        } else {
            SourceAuthority::Project
        }
    }
}
