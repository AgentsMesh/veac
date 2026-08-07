use veac_ir::{TemporalBindingId, TemporalProgramId};
use veac_lang::program::expression::CoreTemporalInputIdentity;
use veac_lang::program::{ClipTemporalProperty, ExecutableTemporalLeaf, ExecutableTemporalSink};

use super::support::{self, SOLID_SOURCE};

fn expression(
    item: veac_ir::ItemId,
    source: &str,
) -> veac_lang::program::expression::CompiledExpression {
    support::compile(
        source,
        [(
            "progress",
            CoreTemporalInputIdentity::Progress { item_id: item },
        )],
    )
}

fn custom(
    item: veac_ir::ItemId,
    source: &str,
    binding: TemporalBindingId,
    request: veac_lang::program::expression::ResidualizationRequest,
) -> ExecutableTemporalLeaf {
    custom_property(
        item,
        ClipTemporalProperty::VisualOpacity,
        source,
        binding,
        request,
    )
}

fn custom_property(
    item: veac_ir::ItemId,
    property: ClipTemporalProperty,
    source: &str,
    binding: TemporalBindingId,
    request: veac_lang::program::expression::ResidualizationRequest,
) -> ExecutableTemporalLeaf {
    ExecutableTemporalLeaf::new(
        ExecutableTemporalSink::clip(item.clone(), property),
        binding,
        expression(item, source),
        request,
    )
}

fn assert_reason(leaves: &[ExecutableTemporalLeaf], reason: &str) {
    let diagnostic = support::failure(SOLID_SOURCE, leaves);
    assert!(
        diagnostic.message.contains(reason),
        "{}",
        diagnostic.message
    );
}

#[test]
fn one_binding_id_cannot_name_two_animation_leaves() {
    let ids = support::ids(SOLID_SOURCE);
    let binding = TemporalBindingId::new("tbd_shared").unwrap();
    let leaves = [
        custom(
            ids.items[0].clone(),
            "progress",
            binding.clone(),
            support::request("bind_a"),
        ),
        custom(
            ids.items[1].clone(),
            "progress",
            binding,
            support::request("bind_b"),
        ),
    ];
    assert_reason(&leaves, "EXECUTABLE_TEMPORAL_BINDING_ID");
}

#[test]
fn one_program_id_cannot_name_different_residual_programs() {
    let ids = support::ids(SOLID_SOURCE);
    let shared = TemporalProgramId::new("tpg_shared").unwrap();
    let mut first = support::request("program_a");
    first.program_id = shared.clone();
    let mut second = support::request("program_b");
    second.program_id = shared;
    let leaves = [
        custom(
            ids.items[0].clone(),
            "progress",
            TemporalBindingId::new("tbd_program_a").unwrap(),
            first,
        ),
        custom(
            ids.items[1].clone(),
            "progress * 2.0",
            TemporalBindingId::new("tbd_program_b").unwrap(),
            second,
        ),
    ];
    assert_reason(&leaves, "EXECUTABLE_TEMPORAL_PROGRAM_ID");
}

#[test]
fn pooled_program_alias_cannot_later_name_different_content() {
    let item = support::ids(SOLID_SOURCE).items[0].clone();
    let shared = TemporalProgramId::new("tpg_alias_shared").unwrap();
    let mut first = support::request("alias_first");
    first.program_id = shared.clone();
    let second = support::request("alias_second");
    let mut conflicting = support::request("alias_conflicting");
    conflicting.program_id = shared;
    let leaves = [
        custom_property(
            item.clone(),
            ClipTemporalProperty::VisualOpacity,
            "progress",
            TemporalBindingId::new("tbd_alias_first").unwrap(),
            first,
        ),
        custom_property(
            item.clone(),
            ClipTemporalProperty::AudioGain,
            "progress * 2.0",
            TemporalBindingId::new("tbd_alias_second").unwrap(),
            second,
        ),
        custom_property(
            item,
            ClipTemporalProperty::AudioPan,
            "progress * 2.0",
            TemporalBindingId::new("tbd_alias_conflicting").unwrap(),
            conflicting,
        ),
    ];
    assert_reason(&leaves, "EXECUTABLE_TEMPORAL_PROGRAM_ID");
}

#[test]
fn one_provenance_id_cannot_alias_different_authored_sites() {
    let ids = support::ids(SOLID_SOURCE);
    let first = support::request("provenance_a");
    let mut second = support::request("provenance_b");
    second.provenance.id = first.provenance.id.clone();
    let leaves = [
        custom(
            ids.items[0].clone(),
            "progress",
            TemporalBindingId::new("tbd_provenance_a").unwrap(),
            first,
        ),
        custom(
            ids.items[1].clone(),
            "progress",
            TemporalBindingId::new("tbd_provenance_b").unwrap(),
            second,
        ),
    ];
    assert_reason(&leaves, "EXECUTABLE_TEMPORAL_PROVENANCE_ID");
}
