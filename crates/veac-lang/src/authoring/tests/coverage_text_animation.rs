use super::coverage_text_enums::text_style;

#[test]
fn every_text_animation_unit_lowers_exactly() {
    use veac_ir::TextGranularity as U;
    for (token, expected) in [
        ("whole", U::Whole),
        ("line", U::Line),
        ("word", U::Word),
        ("grapheme", U::Grapheme),
    ] {
        assert_eq!(
            text_style("unit word;", &format!("unit {token};"))
                .animation
                .unwrap()
                .granularity,
            expected
        );
    }
}
