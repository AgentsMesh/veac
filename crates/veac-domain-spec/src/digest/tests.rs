use super::*;

fn digest(values: Vec<DomainPluginIdentity<'_>>) -> Vec<u8> {
    let mut digest = Sha256::new();
    plugin_identities(&mut digest, values);
    digest.finalize().to_vec()
}

#[test]
fn plugin_identities_are_order_independent_and_value_sensitive() {
    let original = DomainPluginIdentity::new("video.plugin.example.original.v1", "original");
    let alternate = DomainPluginIdentity::new("video.plugin.example.alternate.v1", "alternate");

    assert_eq!(
        digest(vec![original, alternate]),
        digest(vec![alternate, original])
    );

    let changed = DomainPluginIdentity::new("video.plugin.example.alternate.v1", "changed");
    assert_ne!(
        digest(vec![original, alternate]),
        digest(vec![original, changed])
    );
}
