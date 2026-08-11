use std::collections::BTreeMap;

use super::{EntryContract, EntryValueType};
use crate::program::expression::PrimitiveType;
use crate::program::loader::MemoryLoader;
use crate::program::{prepare_host_with_loader, LoadedSource};

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
