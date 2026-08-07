use super::{automatic, set, OutputSentinel, SETS};

#[test]
fn runtime_constructors_preserve_output_terms() {
    let expected = &SETS[0];
    let actual = set(
        std::hint::black_box(expected.context),
        std::hint::black_box(expected.tokens),
    );
    assert_eq!(actual.context, expected.context);
    assert_eq!(actual.tokens, expected.tokens);

    let automatic = automatic(std::hint::black_box(expected.context));
    assert_eq!(automatic.context, expected.context);
    assert_eq!(automatic.tokens, OutputSentinel::AUTOMATIC_TOKENS);
}
