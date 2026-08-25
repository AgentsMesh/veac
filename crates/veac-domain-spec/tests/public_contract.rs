use veac_domain_spec::{
    DomainOperationId, DomainOperationRegistry, DomainOpsetVersion, DomainPluginIdentity,
    DomainType,
};

#[test]
fn public_registry_publishes_the_closed_v8_inventory() {
    let registry = DomainOperationRegistry::standard();
    assert_eq!(registry.version(), DomainOpsetVersion::CURRENT);
    assert_eq!(registry.len(), DomainOperationId::all().count());
    assert_eq!(DomainType::all().count(), 214);
    assert_eq!(
        registry.lookup_name("canvas").unwrap().id(),
        DomainOperationId::Canvas
    );
    assert_eq!(
        registry.lookup_function("canvas").unwrap().id(),
        DomainOperationId::Canvas
    );
}

#[test]
fn plugin_identity_is_typed_and_orderable_without_runtime_dependencies() {
    let first = DomainPluginIdentity::new("video.plugin.a", "digest-a");
    let second = DomainPluginIdentity::new("video.plugin.b", "digest-b");
    assert!(first < second);
    assert_eq!(first.effect_type(), "video.plugin.a");
    assert_eq!(first.digest(), "digest-a");
}
