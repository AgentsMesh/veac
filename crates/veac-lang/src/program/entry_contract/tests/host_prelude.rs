use std::cell::Cell;
use std::collections::BTreeMap;

use super::{EntryContract, EntryValueType};
use crate::program::expression::PrimitiveType;
use crate::program::loader::MemoryLoader;
use crate::program::{prepare_host_with_loader, LoadedSource, SourceAuthority, SourceLoader};

#[test]
fn root_prelude_types_are_unqualified_and_nominal_identity_is_pinned() {
    const ABI: &str = "veac/project-manifest-v1.veac";
    let loader = MemoryLoader::new(BTreeMap::from([(
        ABI.to_owned(),
        "module { export struct ProjectManifest { id: identifier, } }".to_owned(),
    )]));
    let source = "fn workspace() -> ProjectManifest { \
                  ProjectManifest { id: identifier(\"demo\"), } }";
    let entry = || LoadedSource {
        id: "project.veac".to_owned(),
        source: source.to_owned(),
    };
    let contract = EntryContract::new(
        "workspace",
        EntryValueType::nominal_from(ABI, "ProjectManifest"),
    )
    .with_prelude(ABI);
    let prepared = prepare_host_with_loader(entry(), &loader, &contract).unwrap();
    assert_eq!(prepared.sources().len(), 2);
    assert!(prepared.execute(&[]).is_ok());

    let wrong = EntryContract::new(
        "workspace",
        EntryValueType::nominal_from("veac/other.veac", "ProjectManifest"),
    )
    .with_prelude(ABI);
    assert_eq!(
        prepare_host_with_loader(entry(), &loader, &wrong)
            .unwrap_err()
            .as_slice()[0]
            .code,
        "PROGRAM_ENTRY_SIGNATURE"
    );
}

#[test]
fn host_runtime_errors_keep_the_innermost_authored_source() {
    let helper = "module { export fn explode(value: scalar) -> scalar { 1.0 / value } }";
    let loader = MemoryLoader::new(BTreeMap::from([(
        "helper.veac".to_owned(),
        helper.to_owned(),
    )]));
    let entry = LoadedSource {
        id: "project.veac".to_owned(),
        source: "import \"./helper.veac\" as helper; \
                 fn workspace() -> scalar { helper.explode(0.0) }"
            .to_owned(),
    };
    let contract = EntryContract::new(
        "workspace",
        EntryValueType::Primitive(PrimitiveType::Scalar),
    );
    let prepared = prepare_host_with_loader(entry, &loader, &contract).unwrap();
    let error = prepared.execute(&[]).unwrap_err();
    let diagnostic = &error.as_slice()[0];
    assert_eq!(diagnostic.code, "PROGRAM_ENTRY_RUNTIME");
    assert_eq!(diagnostic.path, "helper.veac");
    assert!(helper[diagnostic.span.start..diagnostic.span.end].contains("1.0 / value"));
    assert!(diagnostic.message.contains("called from project.veac"));
}

#[test]
fn host_freezes_implicit_and_explicit_routes_before_resolution() {
    const ABI: &str = "veac/project-manifest-v1.veac";
    struct CountingLoader(Cell<usize>);
    impl SourceLoader for CountingLoader {
        fn load(&self, _: &str, requested: &str) -> Result<LoadedSource, String> {
            self.0.set(self.0.get() + 1);
            Ok(LoadedSource {
                id: requested.to_owned(),
                source: "module { export struct Manifest { id: identifier, } }".to_owned(),
            })
        }

        fn authority(&self, _: &str) -> SourceAuthority {
            SourceAuthority::ReadOnlyDependency
        }
    }
    let loader = CountingLoader(Cell::new(0));
    let entry = LoadedSource {
        id: "project.veac".to_owned(),
        source: format!(
            "import \"{ABI}\" as abi; fn workspace() -> Manifest {{ \
             Manifest {{ id: identifier(\"demo\"), }} }}"
        ),
    };
    let contract = EntryContract::new("workspace", EntryValueType::nominal_from(ABI, "Manifest"))
        .with_prelude(ABI);
    let prepared = prepare_host_with_loader(entry, &loader, &contract).unwrap();
    assert_eq!(loader.0.get(), 1);
    assert_eq!(
        prepared.source_graph().authority(ABI),
        SourceAuthority::ReadOnlyDependency
    );
}

#[test]
fn implicit_prelude_cycles_back_to_the_root_fail_during_freeze() {
    struct CycleLoader;
    impl SourceLoader for CycleLoader {
        fn load(&self, _: &str, requested: &str) -> Result<LoadedSource, String> {
            let (id, source) = if requested == "abi.veac" {
                ("abi.veac", "module { import \"./project.veac\" as root; }")
            } else {
                ("project.veac", "fn workspace() -> int { 1 }")
            };
            Ok(LoadedSource {
                id: id.to_owned(),
                source: source.to_owned(),
            })
        }

        fn authority(&self, _: &str) -> SourceAuthority {
            SourceAuthority::Project
        }
    }
    let contract = EntryContract::new(
        "workspace",
        EntryValueType::Primitive(PrimitiveType::Integer),
    )
    .with_prelude("abi.veac");
    let error = prepare_host_with_loader(
        LoadedSource {
            id: "project.veac".to_owned(),
            source: "fn workspace() -> int { 1 }".to_owned(),
        },
        &CycleLoader,
        &contract,
    )
    .unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_IMPORT_CYCLE");
}
