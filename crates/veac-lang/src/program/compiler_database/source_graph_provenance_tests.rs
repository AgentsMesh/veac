use std::cell::Cell;

use super::source_graph;
use super::CompilerDatabase;
use crate::program::{LoadedSource, SourceAuthority, SourceLoader};

const MODULE: &str = "module { export const time value = 1s; }";

fn root(source: &str) -> LoadedSource {
    LoadedSource {
        id: "root.veac".into(),
        source: source.into(),
    }
}

#[test]
fn one_source_id_cannot_drift_authority_with_identical_bytes() {
    struct AuthorityLoader(Cell<usize>);
    impl SourceLoader for AuthorityLoader {
        fn load(&self, _: &str, _: &str) -> Result<LoadedSource, String> {
            Ok(LoadedSource {
                id: "shared.veac".into(),
                source: MODULE.into(),
            })
        }

        fn authority(&self, _: &str) -> SourceAuthority {
            let call = self.0.get();
            self.0.set(call + 1);
            if call < 2 {
                SourceAuthority::Project
            } else {
                SourceAuthority::ReadOnlyDependency
            }
        }
    }
    let error = source_graph::discover(
        &CompilerDatabase::default(),
        root("module { import \"./a.veac\" as a; import \"./b.veac\" as b; }"),
        &AuthorityLoader(Cell::new(0)),
    )
    .unwrap_err();
    assert_eq!(error[0].code, "PROGRAM_SOURCE_AUTHORITY_COLLISION");
}

#[test]
fn semantic_interface_cache_never_reuses_source_authority() {
    struct FixedLoader(SourceAuthority);
    impl SourceLoader for FixedLoader {
        fn load(&self, _: &str, _: &str) -> Result<LoadedSource, String> {
            Ok(LoadedSource {
                id: "a.veac".into(),
                source: MODULE.into(),
            })
        }

        fn authority(&self, _: &str) -> SourceAuthority {
            self.0
        }
    }
    let database = CompilerDatabase::default();
    let source = root("module { import \"./a.veac\" as a; export fn value() -> time { a.value } }");
    database
        .module_interface(source.clone(), &FixedLoader(SourceAuthority::Project))
        .unwrap();
    let before = database.statistics();
    database
        .module_interface(
            source.clone(),
            &FixedLoader(SourceAuthority::ReadOnlyDependency),
        )
        .unwrap();
    assert_eq!(
        database.statistics().interface_hits,
        before.interface_hits + 1
    );
    let graph = source_graph::discover(
        &database,
        source,
        &FixedLoader(SourceAuthority::ReadOnlyDependency),
    )
    .unwrap();
    assert_eq!(
        graph.authority("a.veac"),
        SourceAuthority::ReadOnlyDependency
    );
}
