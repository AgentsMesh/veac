use super::{FileIdentity, IdentityRegistry};

fn identity(device: i128, inode: u128) -> FileIdentity {
    FileIdentity { device, inode }
}

#[test]
fn registry_accepts_repeated_registration_for_the_same_source_id() {
    let registry = IdentityRegistry::default();
    registry.register("module.veac", identity(1, 2)).unwrap();
    registry.register("module.veac", identity(1, 2)).unwrap();
}

#[test]
fn registry_reports_a_poisoned_identity_map_without_panicking() {
    let registry = IdentityRegistry::default();
    let poisoner = registry.clone();
    assert!(std::thread::spawn(move || {
        let _guard = poisoner.paths.lock().unwrap();
        panic!("poison identity registry for the contract test");
    })
    .join()
    .is_err());

    assert_eq!(
        registry.register("module.veac", identity(1, 2)),
        Err("source identity registry is unavailable".to_owned())
    );
}
