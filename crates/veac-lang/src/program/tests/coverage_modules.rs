use std::collections::BTreeMap;

use super::empty_entry;
use crate::program::loader::MemoryLoader;
use crate::program::{compile_with_loader, LoadedSource, SourceLoader};

fn compile_error(
    id: &str,
    source: String,
    loader: &dyn SourceLoader,
) -> crate::program::Diagnostics {
    compile_with_loader(
        LoadedSource {
            id: id.to_owned(),
            source,
        },
        loader,
    )
    .unwrap_err()
}

#[test]
fn root_source_id_and_entry_kind_are_checked_before_resolution() {
    let loader = MemoryLoader::default();
    let invalid = compile_error("../main.veac", empty_entry(""), &loader);
    assert_eq!(invalid.as_slice()[0].code, "PROGRAM_SOURCE_ID");
    let module = compile_error("main.veac", "module {}".to_owned(), &loader);
    assert_eq!(module.as_slice()[0].code, "PROGRAM_ENTRY_MODULE");
}

#[test]
fn duplicate_alias_cache_and_project_as_module_paths_are_exercised() {
    let loader = MemoryLoader::new(BTreeMap::from([
        ("shared.veac".to_owned(), "module {}".to_owned()),
        ("project.veac".to_owned(), empty_entry("")),
    ]));
    let duplicate = compile_error(
        "main.veac",
        empty_entry("import \"./shared.veac\" as same; import \"./shared.veac\" as same;"),
        &loader,
    );
    assert_eq!(duplicate.as_slice()[0].code, "PROGRAM_IMPORT_ALIAS");

    let cached = compile_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: empty_entry(
                "import \"./shared.veac\" as first; import \"./shared.veac\" as second;",
            ),
        },
        &loader,
    )
    .unwrap();
    assert_eq!(cached.sources().len(), 2);

    let project = compile_error(
        "main.veac",
        empty_entry("import \"./project.veac\" as nested;"),
        &loader,
    );
    assert_eq!(project.as_slice()[0].code, "PROGRAM_IMPORT_PROJECT");
}

struct CollidingLoader;

impl SourceLoader for CollidingLoader {
    fn load(&self, _: &str, requested: &str) -> Result<LoadedSource, String> {
        Ok(LoadedSource {
            id: "same.veac".to_owned(),
            source: format!("module {{ export const text source = \"{requested}\"; }}"),
        })
    }
}

#[test]
fn one_source_id_cannot_resolve_to_different_contents() {
    let source =
        empty_entry("import \"./first.veac\" as first; import \"./second.veac\" as second;");
    let error = compile_error("main.veac", source, &CollidingLoader);
    assert_eq!(error.as_slice()[0].code, "PROGRAM_SOURCE_ID_COLLISION");
}

struct DeepLoader;

impl SourceLoader for DeepLoader {
    fn load(&self, _: &str, requested: &str) -> Result<LoadedSource, String> {
        let id = requested.trim_start_matches("./").to_owned();
        let number = id
            .strip_prefix('m')
            .and_then(|value| value.strip_suffix(".veac"))
            .and_then(|value| value.parse::<usize>().ok())
            .ok_or_else(|| "invalid depth fixture".to_owned())?;
        Ok(LoadedSource {
            id,
            source: format!("module {{ import \"./m{}.veac\" as next; }}", number + 1),
        })
    }
}

#[test]
fn import_graph_depth_is_bounded() {
    let error = compile_error(
        "main.veac",
        empty_entry("import \"./m0.veac\" as first;"),
        &DeepLoader,
    );
    assert_eq!(error.as_slice()[0].code, "PROGRAM_IMPORT_DEPTH");
}

#[test]
fn trusted_compile_boundary_rejects_oversized_custom_loader_modules() {
    let missing = MemoryLoader::default()
        .load("main.veac", "missing.veac")
        .unwrap_err();
    assert!(missing.contains("was not found"));

    let loader = MemoryLoader::new(BTreeMap::from([(
        "large.veac".to_owned(),
        "x".repeat(16 * 1024 * 1024 + 1),
    )]));
    assert!(loader.load("main.veac", "large.veac").unwrap().source.len() > 16 * 1024 * 1024);
    let error = compile_error(
        "main.veac",
        empty_entry("import \"./large.veac\" as large;"),
        &loader,
    );
    assert_eq!(error.as_slice()[0].code, "PROGRAM_SOURCE_LIMIT");

    let entry_error = compile_error(
        "main.veac",
        "x".repeat(16 * 1024 * 1024 + 1),
        &MemoryLoader::default(),
    );
    assert_eq!(entry_error.as_slice()[0].code, "PROGRAM_SOURCE_LIMIT");
}
