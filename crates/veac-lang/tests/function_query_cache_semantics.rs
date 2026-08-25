use std::collections::BTreeMap;

use veac_lang::program::{CompilerDatabase, LoadedSource, SourceLoader};

#[path = "program_functions/support.rs"]
mod support;

#[derive(Default)]
struct Loader(BTreeMap<String, String>);

impl SourceLoader for Loader {
    fn load(&self, _importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let id = requested.trim_start_matches("./").to_owned();
        self.0
            .get(&id)
            .cloned()
            .map(|source| LoadedSource { id, source })
            .ok_or_else(|| format!("missing module `{requested}`"))
    }

    fn authority(&self, _source_id: &str) -> veac_lang::program::SourceAuthority {
        veac_lang::program::SourceAuthority::Project
    }
}

fn entry(declarations: &str, duration: &str) -> LoadedSource {
    LoadedSource {
        id: "main.veac".into(),
        source: support::project_with(declarations, duration),
    }
}

fn cached_and_clean_error(
    database: &CompilerDatabase,
    source: LoadedSource,
    loader: &Loader,
) -> (
    veac_lang::program::Diagnostics,
    veac_lang::program::Diagnostics,
) {
    let cached = database
        .prepare_with_loader(source.clone(), loader)
        .unwrap()
        .execute()
        .unwrap_err();
    let clean = CompilerDatabase::default()
        .prepare_with_loader(source, loader)
        .unwrap()
        .execute()
        .unwrap_err();
    (cached, clean)
}

fn assert_current_default(diagnostics: &veac_lang::program::Diagnostics, source: &LoadedSource) {
    let diagnostic = &diagnostics.as_slice()[0];
    let start = source.source.find("1s / 0.0").unwrap();
    let end = start + "1s / 0.0".len();
    assert_eq!(diagnostic.path, source.id);
    assert!(diagnostic.span.start >= start);
    assert!(diagnostic.span.end <= end);
}

#[test]
fn cached_local_default_uses_the_current_authored_span() {
    let first = "fn duration(value: time =    1s / 0.0) -> time { value }";
    let changed = "fn duration(value: time = 1s / 0.0   ) -> time { value }";
    assert_eq!(first.len(), changed.len());
    assert_eq!(first.find("{ value }"), changed.find("{ value }"));
    assert_eq!(
        first.find("1s / 0.0").unwrap() - changed.find("1s / 0.0").unwrap(),
        3
    );
    let database = CompilerDatabase::default();
    database
        .prepare_with_loader(entry(first, "duration()"), &Loader::default())
        .unwrap();
    let changed = entry(changed, "duration()");
    let (cached, clean) = cached_and_clean_error(&database, changed.clone(), &Loader::default());
    assert_eq!(cached, clean);
    assert_current_default(&cached, &changed);
}

#[test]
fn cached_imported_default_uses_the_current_module_span() {
    let first = "module { export fn duration(value: time =    1s / 0.0) -> time { value } }";
    let changed = "module { export fn duration(value: time = 1s / 0.0   ) -> time { value } }";
    assert_eq!(first.len(), changed.len());
    assert_eq!(first.find("{ value }"), changed.find("{ value }"));
    assert_eq!(
        first.find("1s / 0.0").unwrap() - changed.find("1s / 0.0").unwrap(),
        3
    );
    let root = entry("import \"./timing.veac\" as timing;", "timing.duration()");
    let database = CompilerDatabase::default();
    let loader = Loader(BTreeMap::from([("timing.veac".into(), first.into())]));
    database.prepare_with_loader(root.clone(), &loader).unwrap();
    let changed_module = LoadedSource {
        id: "timing.veac".into(),
        source: changed.into(),
    };
    let changed_loader = Loader(BTreeMap::from([(
        changed_module.id.clone(),
        changed_module.source.clone(),
    )]));
    let (cached, clean) = cached_and_clean_error(&database, root, &changed_loader);
    assert_eq!(cached, clean);
    assert_current_default(&cached, &changed_module);
}
