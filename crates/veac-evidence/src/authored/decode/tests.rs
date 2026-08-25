use std::collections::BTreeMap;

use veac_lang::program::expression::{ExactNumber, Value};
use veac_lang::program::{
    prepare_host_source, prepare_host_with_loader, EntryContract, EntryValueType, LoadedSource,
    SourceAuthority, SourceLoader, TypeRegistry,
};

use super::value::{unknown_variant, Decoder};
use crate::{EVIDENCE_MODULE_ID, EVIDENCE_MODULE_SOURCE};

#[test]
fn primitive_decoder_rejects_wrong_kinds_and_checked_overflow() {
    let types = TypeRegistry::default();
    let decoder = Decoder::new(&types);
    assert!(decoder.text(&Value::Integer(1), "text").is_err());
    assert!(decoder.identifier(&Value::Bool(true), "id").is_err());
    assert!(decoder.bool(&Value::Integer(1), "bool").is_err());
    assert!(decoder.integer(&Value::Bool(true), "int").is_err());
    assert!(decoder.scalar(&Value::Integer(1), "scalar").is_err());
    assert!(decoder.time(&Value::Integer(1), "time").is_err());
    assert!(decoder.u64(&Value::Integer(-1), "u64").is_err());
    assert!(decoder.usize(&Value::Integer(-1), "usize").is_err());
    let unsafe_number = ExactNumber::integer(9_007_199_254_740_992);
    assert!(decoder
        .scalar(&Value::Scalar(unsafe_number), "scalar")
        .is_err());
    let numerator = ExactNumber::integer(i128::from(i64::MAX) + 1);
    assert!(decoder.time(&Value::Time(numerator), "time").is_err());
    let denominator = ExactNumber::new(1, i128::from(u32::MAX) + 1).unwrap();
    assert!(decoder.time(&Value::Time(denominator), "time").is_err());
    let unsafe_time = ExactNumber::integer(i128::from(i64::MAX));
    assert!(decoder.time(&Value::Time(unsafe_time), "time").is_err());
}

#[test]
fn structural_decoder_defends_nominal_identity_and_shape() {
    let types = TypeRegistry::default();
    let decoder = Decoder::new(&types);
    assert!(decoder
        .structure(&Value::Integer(1), "EvidenceSuite", "suite")
        .is_err());
    assert!(decoder
        .variant(&Value::Integer(1), "DiffChannels", "channels")
        .is_err());
    assert!(decoder
        .list_map(&Value::Integer(1), "items", passthrough)
        .is_err());

    let suite = evaluated(
        "fn evidence() -> EvidenceSuite { EvidenceSuite { schema_version: 1, id: identifier(\"x\"), sources: [], samples: [], regions: [], assertions: [], } }",
        "evidence",
        "EvidenceSuite",
    );
    let decoder = Decoder::new(suite.type_registry());
    let fields = decoder
        .structure(suite.value(), "EvidenceSuite", "suite")
        .unwrap();
    assert!(fields.get("absent").is_err());
    assert!(decoder
        .variant(suite.value(), "EvidenceSuite", "suite")
        .is_err());
    assert!(decoder
        .structure(suite.value(), "EvidenceSource", "suite")
        .is_err());
    let empty = TypeRegistry::default();
    assert!(Decoder::new(&empty)
        .structure(suite.value(), "EvidenceSuite", "suite")
        .is_err());

    let channel = evaluated(
        "fn channel() -> DiffChannels { DiffChannels.Rgb }",
        "channel",
        "DiffChannels",
    );
    let decoder = Decoder::new(channel.type_registry());
    assert!(decoder
        .structure(channel.value(), "DiffChannels", "channel")
        .is_err());
    assert_eq!(
        decoder
            .variant(channel.value(), "DiffChannels", "channel")
            .unwrap()
            .name,
        "Rgb"
    );
    assert!(unknown_variant("x", "Example", "Future")
        .to_string()
        .contains("Future"));
}

#[test]
fn local_nominal_values_cannot_cross_the_builtin_abi_boundary() {
    let prepared = prepare_host_source(
        "struct Local { value: int, } fn make() -> Local { Local { value: 1, } }",
        &EntryContract::new("make", EntryValueType::nominal("Local")),
    )
    .unwrap();
    let evaluated = prepared.execute(&[]).unwrap();
    assert!(Decoder::new(evaluated.type_registry())
        .structure(evaluated.value(), "Local", "local")
        .is_err());
}

fn evaluated(source: &str, function: &str, result: &str) -> veac_lang::program::EvaluatedHostEntry {
    let loader = AbiLoader;
    let contract = EntryContract::new(
        function,
        EntryValueType::nominal_from(EVIDENCE_MODULE_ID, result),
    )
    .with_prelude(EVIDENCE_MODULE_ID);
    prepare_host_with_loader(
        LoadedSource {
            id: "test.veac".into(),
            source: source.into(),
        },
        &loader,
        &contract,
    )
    .unwrap()
    .execute(&[])
    .unwrap()
}

fn passthrough(
    _decoder: &Decoder<'_>,
    value: &Value,
    _path: &str,
) -> Result<Value, super::super::EvidenceDecodeError> {
    Ok(value.clone())
}

struct AbiLoader;

impl SourceLoader for AbiLoader {
    fn load(&self, _importer: &str, requested: &str) -> Result<LoadedSource, String> {
        BTreeMap::from([(EVIDENCE_MODULE_ID, EVIDENCE_MODULE_SOURCE)])
            .get(requested)
            .map(|source| LoadedSource {
                id: requested.to_owned(),
                source: (*source).to_owned(),
            })
            .ok_or_else(|| format!("unknown source {requested}"))
    }

    fn authority(&self, _source_id: &str) -> SourceAuthority {
        SourceAuthority::ReadOnlyDependency
    }
}
