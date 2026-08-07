use super::*;

fn digest(values: Vec<veac_ir::PluginEffectDescriptor>) -> Vec<u8> {
    let mut digest = Sha256::new();
    plugin_identities(&mut digest, values);
    digest.finalize().to_vec()
}

#[test]
fn plugin_identities_are_order_independent_and_value_sensitive() {
    let original = veac_ir::plugin_effects()[0];
    let mut alternate = original;
    alternate.effect_type = "video.plugin.example.alternate.v1";
    alternate.digest = "alternate-digest";

    assert_eq!(
        digest(vec![original, alternate]),
        digest(vec![alternate, original])
    );

    let mut changed = alternate;
    changed.digest = "changed-digest";
    assert_ne!(
        digest(vec![original, alternate]),
        digest(vec![original, changed])
    );
}
