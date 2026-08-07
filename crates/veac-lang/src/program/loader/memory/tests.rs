use super::*;

#[test]
fn missing_memory_module_reports_the_request_and_importer() {
    let loader = MemoryLoader::default();
    let error = loader.load("root/main.veac", "./missing.veac").unwrap_err();
    assert!(error.contains("missing.veac"));
    assert!(error.contains("root/main.veac"));
}

#[test]
fn memory_module_load_returns_the_normalized_identity_and_source() {
    let loader = MemoryLoader::new(BTreeMap::from([(
        "root/child.veac".to_owned(),
        "module {}".to_owned(),
    )]));
    let loaded = loader.load("root/main.veac", "./child.veac").unwrap();
    assert_eq!(loaded.id, "root/child.veac");
    assert_eq!(loaded.source, "module {}");
}
