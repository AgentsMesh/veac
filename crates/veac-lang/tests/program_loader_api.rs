use std::collections::BTreeMap;

use veac_lang::program::{compile_with_loader, LoadedSource, SourceLoader};

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
  export const time duration = 1s;
}"#;
    let entry = r#"import "./timing.veac" as timing;
project virtual {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main { layer visual content { item sample {
    source generated transparent;
    record { at 0s; duration ${timing.duration}; }
  } } }
}"#;
    let loader = Loader {
        sources: BTreeMap::from([("timing.veac".to_owned(), module.to_owned())]),
    };
    let compiled = compile_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: entry.to_owned(),
        },
        &loader,
    )
    .unwrap();
    assert_eq!(compiled.root_module(), "main.veac");
    assert!(compiled.expanded_source().contains("duration 1s;"));
    assert_eq!(compiled.sources().len(), 2);
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
    let entry = r#"import "./module.veac" as module;
project root {
  settings {
    timebase 1/1000; canvas 1px by 1px;
    frame-rate 1fps; sample-rate 48000hz;
  }
  entry sequence main; sequence main {}
}"#;
    let error = compile_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: entry.to_owned(),
        },
        &InvalidIdLoader,
    )
    .unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_SOURCE_ID");
}
