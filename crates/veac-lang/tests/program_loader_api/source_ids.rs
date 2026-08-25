use veac_lang::program::{prepare_with_loader, LoadedSource, SourceLoader};

use super::executable_entry;

struct ReturnedId(String);

impl SourceLoader for ReturnedId {
    fn load(&self, _importer: &str, _requested: &str) -> Result<LoadedSource, String> {
        Ok(LoadedSource {
            id: self.0.clone(),
            source: "module {}".to_owned(),
        })
    }

    fn authority(&self, _source_id: &str) -> veac_lang::program::SourceAuthority {
        veac_lang::program::SourceAuthority::Project
    }
}

fn invalid_ids() -> Vec<String> {
    vec![
        "bad:id.veac".to_owned(),
        r"bad\id.veac".to_owned(),
        "bad\u{7f}id.veac".to_owned(),
        "bad\u{85}id.veac".to_owned(),
        "a".repeat(4097),
        ".veac-source.lock".to_owned(),
        "nested/.veac-source.lock".to_owned(),
    ]
}

#[test]
fn custom_loader_roots_obey_the_canonical_source_id_contract() {
    for id in invalid_ids() {
        let error = prepare_with_loader(
            LoadedSource {
                id: id.clone(),
                source: String::new(),
            },
            &ReturnedId("module.veac".to_owned()),
        )
        .unwrap_err();
        assert_eq!(error.as_slice()[0].code, "PROGRAM_SOURCE_ID", "{id:?}");
    }
}

#[test]
fn custom_loader_imports_obey_the_canonical_source_id_contract() {
    for id in invalid_ids() {
        let error = prepare_with_loader(
            LoadedSource {
                id: "main.veac".to_owned(),
                source: executable_entry("import \"./module.veac\" as module;", "1s"),
            },
            &ReturnedId(id.clone()),
        )
        .unwrap_err();
        assert_eq!(error.as_slice()[0].code, "PROGRAM_SOURCE_ID", "{id:?}");
    }
}

#[test]
fn every_successful_source_graph_can_build_its_source_index() {
    let prepared = prepare_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: executable_entry("import \"./module.veac\" as module;", "1s"),
        },
        &ReturnedId("parts/片头.veac".to_owned()),
    )
    .unwrap();
    prepared.source_index().unwrap();
}
