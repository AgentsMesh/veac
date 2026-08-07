use veac_ir::{TemporalParameterId, TemporalType, TemporalValue};
use veac_lang::program::expression::CoreTemporalInputIdentity;
use veac_lang::program::ClipTemporalProperty;

use super::support::{self, SOLID_SOURCE};

fn parameter_leaf(id: &str, value: f64) -> veac_lang::program::ExecutableTemporalLeaf {
    let ids = support::ids(SOLID_SOURCE);
    let parameter_id = TemporalParameterId::new(id).unwrap();
    let expression = support::compile(
        "gain",
        [(
            "gain",
            CoreTemporalInputIdentity::Parameter {
                parameter_id: parameter_id.clone(),
                value_type: TemporalType::Scalar,
            },
        )],
    );
    support::leaf(
        ids.items[0].clone(),
        ClipTemporalProperty::VisualOpacity,
        expression,
        "identity",
    )
    .bind_parameter(parameter_id, TemporalValue::Scalar { value })
}

fn digest(leaf: veac_lang::program::ExecutableTemporalLeaf) -> String {
    support::execute(SOLID_SOURCE, &[leaf])
        .executable
        .digests
        .declared_inputs_sha256
}

#[test]
fn manifest_digest_tracks_typed_declarations_but_not_runtime_values() {
    let base = digest(parameter_leaf("tpm_gain_a", 0.25));
    assert_eq!(base, digest(parameter_leaf("tpm_gain_a", 0.75)));
    assert_ne!(base, digest(parameter_leaf("tpm_gain_b", 0.25)));
    let empty = support::execute(SOLID_SOURCE, &[])
        .executable
        .digests
        .declared_inputs_sha256;
    assert_ne!(base, empty);
    assert_eq!(base.len(), 64);
    assert!(base
        .bytes()
        .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value)));
}

#[test]
fn leaf_injection_order_does_not_change_canonical_output() {
    let ids = support::ids(SOLID_SOURCE);
    let leaves = ids
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let expression = support::compile(
                "progress",
                [(
                    "progress",
                    CoreTemporalInputIdentity::Progress {
                        item_id: item.clone(),
                    },
                )],
            );
            support::leaf(
                item.clone(),
                ClipTemporalProperty::VisualOpacity,
                expression,
                &format!("order_{index}"),
            )
        })
        .collect::<Vec<_>>();
    assert!(leaves[0].sink().partial_cmp(leaves[1].sink()).is_some());
    let forward = support::execute(SOLID_SOURCE, &leaves);
    let reverse = support::execute(SOLID_SOURCE, &[leaves[1].clone(), leaves[0].clone()]);
    assert_eq!(
        forward.executable.digests.declared_inputs_sha256,
        reverse.executable.digests.declared_inputs_sha256
    );
    assert_eq!(
        veac_ir::canonical_json(&forward).unwrap(),
        veac_ir::canonical_json(&reverse).unwrap()
    );
}

#[test]
fn prepared_build_reuses_the_same_typed_leaf_without_state_leakage() {
    let ids = support::ids(SOLID_SOURCE);
    let expression = support::compile(
        "progress",
        [(
            "progress",
            CoreTemporalInputIdentity::Progress {
                item_id: ids.items[0].clone(),
            },
        )],
    );
    let leaf = support::leaf(
        ids.items[0].clone(),
        ClipTemporalProperty::VisualOpacity,
        expression,
        "repeated_build",
    );
    let prepared = veac_lang::program::prepare_source(SOLID_SOURCE)
        .unwrap()
        .with_temporal_leaf(leaf);
    assert_eq!(prepared.temporal_leaves().len(), 1);
    let first = prepared.execute().unwrap();
    let second = prepared.execute().unwrap();
    assert_eq!(
        veac_ir::canonical_json(first.envelope()).unwrap(),
        veac_ir::canonical_json(second.envelope()).unwrap()
    );
}
