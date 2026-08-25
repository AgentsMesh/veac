use std::collections::BTreeMap;

use super::*;

struct RoutedLoader {
    routes: BTreeMap<String, String>,
}

impl SourceLoader for RoutedLoader {
    fn load(&self, _importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let id = self
            .routes
            .get(requested)
            .ok_or_else(|| format!("missing route {requested}"))?;
        Ok(LoadedSource {
            id: id.clone(),
            source: "module { export struct Foreign {} }".to_owned(),
        })
    }

    fn authority(&self, _source_id: &str) -> veac_lang::program::SourceAuthority {
        veac_lang::program::SourceAuthority::Project
    }
}

fn root() -> LoadedSource {
    LoadedSource {
        id: "main.veac".to_owned(),
        source: r#"module {
  import "./a.veac" as first;
  import "./b.veac" as second;
  export fn first_value(value: first.Foreign) -> first.Foreign { value }
  export fn second_value(value: second.Foreign) -> second.Foreign { value }
}"#
        .to_owned(),
    }
}

fn loader(first: &str, second: &str) -> RoutedLoader {
    RoutedLoader {
        routes: BTreeMap::from([
            ("./a.veac".to_owned(), first.to_owned()),
            ("./b.veac".to_owned(), second.to_owned()),
        ]),
    }
}

#[test]
fn interface_cache_binds_import_paths_to_resolved_source_ids() {
    let database = CompilerDatabase::default();
    let first = database
        .module_interface(root(), &loader("x.veac", "y.veac"))
        .unwrap();
    let second = database
        .module_interface(root(), &loader("y.veac", "x.veac"))
        .unwrap();
    let first_types = first
        .functions
        .iter()
        .map(|function| {
            (
                function.name.clone(),
                function.parameters[0].value_type.clone(),
            )
        })
        .collect::<Vec<_>>();
    let second_types = second
        .functions
        .iter()
        .map(|function| {
            (
                function.name.clone(),
                function.parameters[0].value_type.clone(),
            )
        })
        .collect::<Vec<_>>();
    assert_ne!(first_types, second_types);
    assert_eq!(database.statistics().interface_hits, 0);
    assert_eq!(database.statistics().interface_misses, 2);
}
