use super::*;

#[test]
fn published_identity_is_stable_complete_and_valid() {
    let first = render_implementation_identity();
    let second = render_implementation_identity();
    assert_eq!(first, second);
    first.digest.validate().unwrap();
    assert_eq!(first.source_sha256.len(), 64);
    assert_eq!(first.build_sha256.len(), 64);
    assert!(!current().plugins.is_empty());
}

#[test]
fn every_render_semantic_axis_changes_the_identity() {
    assert_changed(|value| value.source = "00");
    assert_changed(|value| value.build = "11");
    assert_changed(|value| value.render += 1);
    assert_changed(|value| value.core += 1);
    assert_changed(|value| value.domain += 1);
    assert_changed(|value| value.temporal += 1);
    assert_changed(|value| value.adapter += 1);
    assert_changed(|value| value.plugins[0].effect_type = "changed");
    assert_changed(|value| value.plugins[0].descriptor_digest = "changed");
    assert_changed(|value| value.plugins[0].implementation = "changed");
}

fn assert_changed(change: impl FnOnce(&mut Inputs<'_>)) {
    let expected = calculate(&current());
    let mut value = current();
    change(&mut value);
    assert_ne!(calculate(&value), expected);
}
