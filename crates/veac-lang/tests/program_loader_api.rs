use std::collections::BTreeMap;

use veac_lang::program::{build_with_loader, prepare_with_loader, LoadedSource, SourceLoader};

#[path = "program_functions/support.rs"]
mod support;

#[path = "program_loader_api/budgets.rs"]
mod budgets;
#[path = "program_loader_api/filesystem.rs"]
mod filesystem;
#[path = "program_loader_api/source_ids.rs"]
mod source_ids;

#[derive(Default)]
struct Loader {
    sources: BTreeMap<String, String>,
}

impl SourceLoader for Loader {
    fn load(&self, _importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let id = requested.trim_start_matches("./").to_owned();
        self.sources
            .get(&id)
            .cloned()
            .map(|source| LoadedSource { id, source })
            .ok_or_else(|| "missing test module".to_owned())
    }
}

#[test]
fn external_loaders_can_compile_virtual_source_graphs() {
    let module = r#"module {
  export fn duration() -> time { 1s }
}"#;
    let entry = executable_entry("import \"./timing.veac\" as timing;", "timing.duration()");
    let loader = Loader {
        sources: BTreeMap::from([("timing.veac".to_owned(), module.to_owned())]),
    };
    let built = build_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: entry,
        },
        &loader,
    )
    .unwrap();
    assert_eq!(built.root_module(), "main.veac");
    assert_eq!(support::result_duration(&built), "1s");
    assert_eq!(built.sources().len(), 2);
}

struct InvalidIdLoader;

impl SourceLoader for InvalidIdLoader {
    fn load(&self, _importer: &str, _requested: &str) -> Result<LoadedSource, String> {
        Ok(LoadedSource {
            id: "../escaped.veac".to_owned(),
            source: "module {}".to_owned(),
        })
    }
}

#[test]
fn public_loaders_cannot_return_escaping_source_ids() {
    let entry = executable_entry("import \"./module.veac\" as module;", "1s");
    let error = prepare_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: entry,
        },
        &InvalidIdLoader,
    )
    .unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_SOURCE_ID");
}

fn executable_entry(declarations: &str, duration: &str) -> String {
    support::project_with(declarations, duration)
}
