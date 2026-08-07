use std::fmt::Debug;

use crate::authoring::{
    AudioSelection, HardwareSelection, HlsH264ProfileDecl, OutputFormat, OutputSentinel,
};
use crate::program::expression::{BuiltinFunction, PrimitiveType, UnitSuffix};
use veac_ir::TextGranularity;

use super::SyntaxToken;

#[test]
fn one_contract_round_trips_tokens_across_language_layers() {
    round_trip::<BuiltinFunction>();
    round_trip::<PrimitiveType>();
    round_trip::<UnitSuffix>();
    round_trip::<OutputFormat>();
    round_trip::<HlsH264ProfileDecl>();
    round_trip::<OutputSentinel>();
    round_trip::<AudioSelection>();
    round_trip::<HardwareSelection>();
    round_trip::<TextGranularity>();
}

#[test]
fn unknown_tokens_remain_closed_in_every_layer() {
    for rejected in ["", "unknown", "AUTO", "h266", "milliseconds"] {
        assert_eq!(BuiltinFunction::parse(rejected), None);
        assert_eq!(PrimitiveType::parse(rejected), None);
        assert_eq!(UnitSuffix::parse(rejected), None);
        assert_eq!(OutputFormat::parse(rejected), None);
        assert_eq!(OutputSentinel::parse(rejected), None);
        assert_eq!(AudioSelection::parse(rejected), None);
        assert_eq!(HardwareSelection::parse(rejected), None);
    }
    assert_eq!(AudioSelection::parse("auto"), None);
    assert_eq!(HardwareSelection::parse("none"), None);
}

#[test]
fn composite_selections_publish_their_complete_token_sets() {
    assert_eq!(AudioSelection::ALL.len(), 7);
    assert_eq!(AudioSelection::TOKENS[0], OutputSentinel::None.as_str());
    assert_eq!(HardwareSelection::ALL.len(), 6);
    assert_eq!(HardwareSelection::TOKENS[..2], ["auto", "software"]);
}

fn round_trip<T: SyntaxToken + Debug>() {
    assert_eq!(T::ALL.len(), T::TOKENS.len());
    assert_eq!(T::TOKENS.len(), T::ENTRIES.len());
    for ((entry_token, value), token) in T::ENTRIES.iter().zip(T::TOKENS) {
        assert_eq!(entry_token, token);
        assert_eq!(value.as_str(), *token);
        assert_eq!(T::parse(token), Some(*value));
    }
    assert_eq!(T::parse("__veac_unknown_syntax_token__"), None);
}
