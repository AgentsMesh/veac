use std::cell::Cell;

use super::*;
use crate::program::{LoadedSource, SourceAuthority, SourceLoader};

const SHARED: &str = "module { export fn value() -> time { 1s } }";

#[test]
fn repeated_route_loads_once_and_reuses_the_frozen_resolution() {
    struct CountingLoader(Cell<usize>);
    impl SourceLoader for CountingLoader {
        fn load(&self, _: &str, _: &str) -> Result<LoadedSource, String> {
            self.0.set(self.0.get() + 1);
            Ok(LoadedSource {
                id: "shared.veac".into(),
                source: SHARED.into(),
            })
        }

        fn authority(&self, _: &str) -> SourceAuthority {
            SourceAuthority::Project
        }
    }
    let loader = CountingLoader(Cell::new(0));
    let root = LoadedSource {
        id: "root.veac".into(),
        source: "module { import \"./shared.veac\" as a; import \"./shared.veac\" as b; }".into(),
    };
    let graph = super::source_graph::discover(&CompilerDatabase::default(), root, &loader).unwrap();
    assert_eq!(loader.0.get(), 1);
    assert_eq!(
        graph.resolved_id("root.veac", "./shared.veac"),
        Some("shared.veac")
    );
}

#[test]
fn one_source_id_cannot_change_authority_between_routes() {
    struct DriftingLoader(Cell<usize>);
    impl SourceLoader for DriftingLoader {
        fn load(&self, _: &str, _: &str) -> Result<LoadedSource, String> {
            Ok(LoadedSource {
                id: "shared.veac".into(),
                source: SHARED.into(),
            })
        }

        fn authority(&self, source_id: &str) -> SourceAuthority {
            if source_id != "shared.veac" {
                return SourceAuthority::Project;
            }
            let call = self.0.get();
            self.0.set(call + 1);
            if call == 0 {
                SourceAuthority::Project
            } else {
                SourceAuthority::ReadOnlyDependency
            }
        }
    }
    let root = LoadedSource {
        id: "root.veac".into(),
        source: "module { import \"./a.veac\" as a; import \"./b.veac\" as b; }".into(),
    };
    let errors = super::source_graph::discover(
        &CompilerDatabase::default(),
        root,
        &DriftingLoader(Cell::new(0)),
    )
    .unwrap_err();
    assert_eq!(errors[0].code, "PROGRAM_SOURCE_AUTHORITY_COLLISION");
}
