use super::*;
use crate::program::loader::{LoadedSource, SourceAuthority, SourceLoader};
use crate::program::PreparedSourceGraph;

struct FnLoader(fn(&str, &str) -> Result<LoadedSource, String>);

impl SourceLoader for FnLoader {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        (self.0)(importer, requested)
    }

    fn authority(&self, _source_id: &str) -> crate::program::SourceAuthority {
        crate::program::SourceAuthority::Project
    }
}

fn discover(
    root: LoadedSource,
    loader: &dyn SourceLoader,
) -> Result<PreparedSourceGraph, Vec<crate::program::Diagnostic>> {
    super::source_graph::discover(&CompilerDatabase::default(), root, loader)
}

fn root(source: &str) -> LoadedSource {
    LoadedSource {
        id: "root.veac".into(),
        source: source.into(),
    }
}

fn errors(root: LoadedSource, loader: &dyn SourceLoader) -> Vec<crate::program::Diagnostic> {
    match discover(root, loader) {
        Ok(_) => panic!("source graph discovery unexpectedly succeeded"),
        Err(errors) => errors,
    }
}

fn load_a(_: &str, _: &str) -> Result<LoadedSource, String> {
    Ok(LoadedSource {
        id: "a.veac".into(),
        source: "module { export const time value = 1s; }".into(),
    })
}

#[test]
fn invalid_root_id_is_rejected_before_loading() {
    let invalid = LoadedSource {
        id: "../root.veac".into(),
        source: "module {}".into(),
    };
    let loader = FnLoader(|_, _| Err("loader must not run".into()));
    let errors = errors(invalid, &loader);
    assert_eq!(errors[0].code, "PROGRAM_SOURCE_ID");
    assert_eq!(errors[0].path, "../root.veac");
}

#[test]
fn import_load_failure_keeps_importer_and_requested_context() {
    let loader = FnLoader(|_, requested| Err(format!("missing {requested}")));
    let errors = errors(
        root("module { import \"./missing.veac\" as missing; }"),
        &loader,
    );
    assert_eq!(errors[0].code, "PROGRAM_IMPORT_LOAD");
    assert_eq!(errors[0].path, "root.veac");
    assert!(errors[0].message.contains("./missing.veac"));
}

#[test]
fn invalid_import_source_id_is_rejected() {
    let loader = FnLoader(|_, _| {
        Ok(LoadedSource {
            id: "../outside.veac".into(),
            source: "module {}".into(),
        })
    });
    let errors = errors(
        root("module { import \"./outside.veac\" as outside; }"),
        &loader,
    );
    assert_eq!(errors[0].code, "PROGRAM_SOURCE_ID");
    assert_eq!(errors[0].path, "root.veac");
}

#[test]
fn import_cycles_are_reported_with_the_full_source_chain() {
    let loader = FnLoader(|_, requested| {
        let (id, source) = match requested {
            "./a.veac" => ("a.veac", "module { import \"./root.veac\" as root; }"),
            "./root.veac" => ("root.veac", "module {}"),
            _ => return Err(format!("missing {requested}")),
        };
        Ok(LoadedSource {
            id: id.into(),
            source: source.into(),
        })
    });
    let errors = errors(root("module { import \"./a.veac\" as a; }"), &loader);
    assert_eq!(errors[0].code, "PROGRAM_IMPORT_CYCLE");
    assert!(errors[0]
        .message
        .contains("root.veac -> a.veac -> root.veac"));
}

#[test]
fn source_id_collision_rejects_different_contents_for_one_id() {
    let loader = FnLoader(|_, requested| {
        let source = match requested {
            "./a.veac" => "module { export fn first() -> time { 1s } }",
            "./b.veac" => "module { export fn second() -> time { 2s } }",
            _ => return Err(format!("missing {requested}")),
        };
        Ok(LoadedSource {
            id: "shared.veac".into(),
            source: source.into(),
        })
    });
    let errors = errors(
        root("module { import \"./a.veac\" as a; import \"./b.veac\" as b; }"),
        &loader,
    );
    assert_eq!(errors[0].code, "PROGRAM_SOURCE_ID_COLLISION");
    assert!(errors[0].message.contains("shared.veac"));
}

#[test]
fn import_depth_limit_is_enforced_before_the_65th_module() {
    let loader = FnLoader(|_, requested: &str| {
        let id = requested.trim_start_matches("./").to_owned();
        let number = id.trim_end_matches(".veac").parse::<usize>().unwrap();
        Ok(LoadedSource {
            id,
            source: format!("module {{ import \"./{}.veac\" as next; }}", number + 1),
        })
    });
    let errors = errors(
        LoadedSource {
            id: "0.veac".into(),
            source: "module { import \"./1.veac\" as next; }".into(),
        },
        &loader,
    );
    assert_eq!(errors[0].code, "PROGRAM_IMPORT_DEPTH");
}

#[test]
fn captured_snapshot_rejects_unknown_import_routes() {
    let loader = FnLoader(load_a);
    let snapshot = discover(root("module { import \"./a.veac\" as a; }"), &loader).unwrap();
    let error = snapshot.load("root.veac", "./missing.veac").unwrap_err();
    assert!(error.contains("./missing.veac"));
}

#[test]
fn captured_snapshot_loads_the_exact_resolved_source() {
    let loader = FnLoader(load_a);
    let snapshot = discover(root("module { import \"./a.veac\" as a; }"), &loader).unwrap();
    let loaded = snapshot.load("root.veac", "./a.veac").unwrap();
    assert_eq!(loaded.id, "a.veac");
    assert!(loaded.source.contains("value"));
}

#[test]
fn captured_snapshot_preserves_authority_and_fails_closed_for_unknown_ids() {
    struct DependencyLoader;

    impl SourceLoader for DependencyLoader {
        fn load(&self, _: &str, _: &str) -> Result<LoadedSource, String> {
            load_a("", "")
        }

        fn authority(&self, _: &str) -> SourceAuthority {
            SourceAuthority::ReadOnlyDependency
        }
    }

    let snapshot = discover(
        root("module { import \"./a.veac\" as a; }"),
        &DependencyLoader,
    )
    .unwrap();
    assert_eq!(
        snapshot.authority("a.veac"),
        SourceAuthority::ReadOnlyDependency
    );
    assert_eq!(
        snapshot.authority("missing.veac"),
        SourceAuthority::ReadOnlyDependency
    );
}
