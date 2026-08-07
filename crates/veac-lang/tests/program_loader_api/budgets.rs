use std::fmt::Write;

use veac_lang::program::{prepare_source, prepare_with_loader, LoadedSource, SourceLoader};

use super::executable_entry;

struct GeneratedModules;

struct LargeModules;

impl SourceLoader for GeneratedModules {
    fn load(&self, _importer: &str, requested: &str) -> Result<LoadedSource, String> {
        Ok(LoadedSource {
            id: requested.trim_start_matches("./").to_owned(),
            source: "module {}".to_owned(),
        })
    }
}

impl SourceLoader for LargeModules {
    fn load(&self, _importer: &str, requested: &str) -> Result<LoadedSource, String> {
        Ok(LoadedSource {
            id: requested.trim_start_matches("./").to_owned(),
            source: large_module(),
        })
    }
}

fn large_module() -> String {
    const BYTES: usize = 16 * 1024 * 1024;
    const PREFIX: &str = "module { /*";
    const SUFFIX: &str = "*/ }";
    let available = BYTES - PREFIX.len() - SUFFIX.len();
    let mut source = String::with_capacity(BYTES);
    source.push_str(PREFIX);
    source.extend(std::iter::repeat_n('\u{1f600}', available / 4));
    source.extend(std::iter::repeat_n(' ', available % 4));
    source.push_str(SUFFIX);
    assert_eq!(source.len(), BYTES);
    source
}

fn entry_with_imports(count: usize) -> String {
    let mut source = String::new();
    for index in 0..count {
        writeln!(source, "import \"./m{index}.veac\" as m{index};").unwrap();
    }
    source.push_str(&executable_entry("", "1s"));
    source
}

#[test]
fn public_compile_boundary_enforces_the_1024_module_budget() {
    let error = prepare_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: entry_with_imports(1024),
        },
        &GeneratedModules,
    )
    .unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_MODULE_LIMIT");
}

#[test]
fn public_compile_boundary_enforces_the_64_mib_graph_budget() {
    let error = prepare_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: entry_with_imports(4),
        },
        &LargeModules,
    )
    .unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_SOURCE_GRAPH_LIMIT");
}

#[test]
fn public_compile_boundary_enforces_the_one_million_token_budget() {
    let source = "a ".repeat(1_000_000);
    let error = prepare_source(&source).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_TOKEN_LIMIT");
}

#[test]
fn prepare_source_rejects_oversized_input_before_parsing() {
    let source = " ".repeat(16 * 1024 * 1024 + 1);
    let error = prepare_source(&source).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_SOURCE_LIMIT");
}
