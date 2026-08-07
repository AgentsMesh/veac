use super::{CanonicalRole, ControlUse, ControlWord, GrammarPosition, LanguageLayer};

#[test]
fn runtime_constructor_preserves_the_typed_control_contract() {
    let usage = ControlUse::new(
        std::hint::black_box(ControlWord::parse("module").unwrap()),
        std::hint::black_box(GrammarPosition::ModuleDeclaration),
        std::hint::black_box(CanonicalRole::DeclarationIntroducer),
    );
    assert_eq!(usage.as_str(), "module");
    assert_eq!(usage.layer(), LanguageLayer::StaticProgram);
}
