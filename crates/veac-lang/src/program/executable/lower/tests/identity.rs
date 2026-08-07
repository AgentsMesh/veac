use super::super::id::stable;

#[test]
fn complete_owner_paths_isolate_equal_leaf_keys() {
    let left = stable("item", &["project", "left", "visual", "title"]);
    let right = stable("item", &["project", "right", "visual", "title"]);
    assert_ne!(left, right);
}

#[test]
fn entity_kinds_are_part_of_the_digest_domain() {
    let path = ["project", "main", "content"];
    let track = stable("track", &path);
    let item = stable("item", &path);
    assert_ne!(track, item);
}

#[test]
fn material_ids_have_an_isolated_kind_domain_and_prefix() {
    let path = ["project", "shared"];
    let material = veac_ir::MaterialId::from_digest(stable("material", &path));
    let item = veac_ir::ItemId::from_digest(stable("item", &path));
    assert!(material.as_str().starts_with("med_"));
    assert_ne!(&material.as_str()[4..], &item.as_str()[4..]);
}

#[test]
fn stable_suffix_is_one_lowercase_sha256_digest() {
    let value =
        veac_ir::ItemId::from_digest(stable("item", &["project", "main", "visual", "clip"]));
    assert_eq!(
        value.as_str(),
        "itm_7477103ee2de4afa0dcadad3f50d41d4d58d2d165d9aa9de255fb7bb6c56fceb"
    );
    let suffix = &value.as_str()[veac_ir::ItemId::PREFIX.len()..];
    assert_eq!(suffix.len(), 64);
    assert!(suffix
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
}
