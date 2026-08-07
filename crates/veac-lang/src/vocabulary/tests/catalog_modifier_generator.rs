use super::{set, SETS};

#[test]
fn runtime_set_constructor_preserves_modifier_terms() {
    let expected = &SETS[0];
    let actual = set(
        std::hint::black_box(expected.context),
        std::hint::black_box(expected.tokens),
    );
    assert_eq!(actual.context, expected.context);
    assert_eq!(actual.tokens, expected.tokens);
}
