use super::*;
use crate::vocabulary::{accepts_expression_name, ExpressionNameKind};

#[test]
fn impl_and_self_have_precise_static_program_roles() {
    let vocabulary = language_spec().vocabulary;
    assert_use(
        vocabulary.lookup("impl").unwrap(),
        GrammarPosition::StaticDeclaration,
        CanonicalRole::DeclarationIntroducer,
    );
    assert_use(
        vocabulary.lookup("self").unwrap(),
        GrammarPosition::MethodReceiver,
        CanonicalRole::ReceiverBinding,
    );
    for kind in ExpressionNameKind::ALL {
        assert!(!accepts_expression_name("self", kind));
    }
}
