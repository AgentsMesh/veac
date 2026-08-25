use crate::{Effect, FunctionEffect, Stage};

#[test]
fn effect_and_function_effect_domains_are_closed_and_displayable() {
    let effects = [Effect::Pure, Effect::LocalMutation, Effect::GraphEmit];
    assert_eq!(effects[0].cmp(&effects[1]), std::cmp::Ordering::Less);
    assert_eq!(effects[1].cmp(&effects[2]), std::cmp::Ordering::Less);
    for (effect, token) in FunctionEffect::ALL.into_iter().zip(FunctionEffect::TOKENS) {
        assert_eq!(effect.as_str(), *token);
        assert_eq!(FunctionEffect::parse(token), Some(effect));
        assert_eq!(effect.to_string(), *token);
    }
    assert_eq!(FunctionEffect::parse("unknown"), None);
}

#[test]
fn stage_join_is_the_lattice_maximum_for_every_pair() {
    let stages = [Stage::Const, Stage::Build, Stage::Temporal];
    for left in stages {
        for right in stages {
            let expected = stages
                .into_iter()
                .find(|candidate| *candidate >= left && *candidate >= right)
                .unwrap();
            assert_eq!(left.join(right), expected);
            assert_eq!(right.join(left), expected);
        }
    }
}
