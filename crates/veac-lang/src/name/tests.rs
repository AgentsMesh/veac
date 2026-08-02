use super::{is_name, is_qualified_name};

#[test]
fn canonical_names_are_ascii_unambiguous_and_bounded() {
    for valid in ["a", "_private", "title-card", "name_2"] {
        assert!(is_name(valid), "{valid}");
    }
    for invalid in [
        "",
        "2fast",
        "-lead",
        "trail-",
        "two--dash",
        "a.b",
        "标题",
        "true",
        "false",
    ] {
        assert!(!is_name(invalid), "{invalid}");
    }
    assert!(is_name(&"a".repeat(128)));
    assert!(!is_name(&"a".repeat(129)));
}

#[test]
fn qualified_names_validate_every_segment() {
    assert!(is_qualified_name("brand.title-card"));
    assert!(is_qualified_name("_theme._duration"));
    for invalid in ["brand..title", ".title", "brand.标题", "brand.true"] {
        assert!(!is_qualified_name(invalid), "{invalid}");
    }
}
