use super::*;

#[test]
fn name_lookup_is_confined_to_the_name_first_key_range() {
    let values = BTreeMap::from([
        (preset_key(PresetKind::TextStyle, "alpha"), ()),
        (preset_key(PresetKind::DeliveryProfile, "omega"), ()),
    ]);
    assert!(preset_name_exists(&values, "alpha"));
    assert!(preset_name_exists(&values, "omega"));
    assert!(!preset_name_exists(&values, "middle"));
}

#[test]
fn name_lookup_remains_exact_in_a_wide_map() {
    let values = (0..16_384)
        .map(|index| {
            (
                preset_key(PresetKind::TextStyle, format!("style-{index:05}")),
                (),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert!(preset_name_exists(&values, "style-16383"));
    assert!(!preset_name_exists(&values, "style-16384"));
}
