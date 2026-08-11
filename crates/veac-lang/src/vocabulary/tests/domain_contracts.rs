use std::collections::BTreeSet;

use super::super::{language_spec, DomainValueShapeSpec, LanguageSpec, SyntaxVocabulary};

#[test]
fn current_standard_library_maps_the_complete_closed_opset() {
    let spec = language_spec();
    spec.validate().unwrap();
    assert_eq!(spec.domain_opset.types.len(), 214);
    assert_eq!(spec.domain_opset.operations.len(), 582);
    assert_eq!(spec.standard_library.types.len(), 214);
    assert_eq!(spec.standard_library.free_functions.len(), 559);
    assert_eq!(spec.standard_library.methods.len(), 23);

    let operations = spec
        .domain_opset
        .operations
        .iter()
        .map(|operation| operation.opcode)
        .collect::<BTreeSet<_>>();
    let callables = spec
        .standard_library
        .free_functions
        .iter()
        .map(|function| function.operation_opcode)
        .chain(
            spec.standard_library
                .methods
                .iter()
                .map(|method| method.operation_opcode),
        )
        .collect::<BTreeSet<_>>();
    assert_eq!(callables, operations);
    assert_eq!(spec.standard_library.methods[0].receiver.name, "Project");
    assert_eq!(spec.standard_library.methods[0].name, "entry");
    assert!(spec
        .standard_library
        .free_functions
        .iter()
        .any(|value| value.name == "video_resource"));
    assert!(spec
        .standard_library
        .methods
        .iter()
        .any(|value| { value.receiver.name == "Project" && value.name == "with_resource" }));
    let resource = spec
        .domain_opset
        .types
        .iter()
        .find(|value| value.name == "Resource")
        .unwrap();
    assert!(!resource.container);
    assert!(spec
        .standard_library
        .types
        .iter()
        .any(|value| value.name == "Resource"));
    assert!(spec
        .standard_library
        .methods
        .iter()
        .any(|value| value.receiver.name == "Item" && value.name == "with_visual"));
    assert!(spec
        .standard_library
        .methods
        .iter()
        .any(|value| value.receiver.name == "Sequence" && value.name == "with_relation"));
}

#[test]
fn standard_library_names_do_not_change_the_syntax_vocabulary() {
    let baseline = SyntaxVocabulary::current();
    let spec = language_spec();
    assert_eq!(spec.vocabulary, baseline);
    assert_eq!(spec.vocabulary.entries.len(), 92);
    assert!(spec.vocabulary.lexer_keywords.is_empty());
    assert!(spec.vocabulary.lookup("Canvas").is_none());
    assert!(spec.vocabulary.lookup("with_sequence").is_none());
}

#[test]
fn domain_opset_validation_rejects_identity_and_registry_drift() {
    let mut digest = language_spec();
    digest.domain_opset.registry_digest = "00".repeat(32);
    assert_invalid(&digest, "registry digest");

    let mut missing = language_spec();
    missing.domain_opset.operations.pop();
    assert_invalid(&missing, "every operation");

    let mut reordered = language_spec();
    reordered.domain_opset.operations.swap(0, 1);
    assert_invalid(&reordered, "ordered by opcode");

    let mut stale = language_spec();
    stale.domain_opset.operations[0].contract.ordered_operands[0].name = "stale".into();
    assert_invalid(&stale, "stale contract");
}

#[test]
fn standard_library_validation_rejects_binding_and_contract_drift() {
    let mut receiver = language_spec();
    receiver.standard_library.methods[0].receiver.name = "Sequence".into();
    assert_invalid(&receiver, "receiver name and opcode disagree");

    let mut opcode = language_spec();
    opcode.standard_library.free_functions[0].operation_opcode = 0xffff;
    assert_invalid(&opcode, "numeric operation contract");

    let mut contract = language_spec();
    contract.standard_library.methods[0].contract.effect = super::super::DomainEffectSpec::Pure;
    assert_invalid(&contract, "numeric operation contract");

    let mut missing = language_spec();
    missing.standard_library.methods.pop();
    assert_invalid(&missing, "every domain operation");
}

#[test]
fn every_published_shape_names_its_closed_type_and_operand_axis() {
    let spec = language_spec();
    let types = spec
        .domain_opset
        .types
        .iter()
        .map(|value| (value.opcode, value.name.as_str()))
        .collect::<BTreeSet<_>>();
    for operation in &spec.domain_opset.operations {
        for operand in &operation.contract.ordered_operands {
            assert_shape(&operand.shape, &types);
        }
        assert_shape(&operation.contract.result, &types);
    }
}

#[test]
fn strict_json_rejects_unknown_nested_contract_fields() {
    let mut value = serde_json::to_value(language_spec()).unwrap();
    value["domain_opset"]["operations"][0]["contract"]["unknown"] = true.into();
    assert!(serde_json::from_value::<LanguageSpec>(value).is_err());
}

fn assert_shape(shape: &DomainValueShapeSpec, types: &BTreeSet<(u16, &str)>) {
    match shape {
        DomainValueShapeSpec::Primitive { name } | DomainValueShapeSpec::PrimitiveList { name } => {
            assert!(!name.is_empty())
        }
        DomainValueShapeSpec::Domain {
            type_opcode,
            type_name,
        }
        | DomainValueShapeSpec::DomainList {
            type_opcode,
            type_name,
        } => assert!(types.contains(&(*type_opcode, type_name))),
    }
}

fn assert_invalid(value: &LanguageSpec, message: &str) {
    let error = value.validate().unwrap_err().to_string();
    assert!(error.contains(message), "unexpected error: {error}");
}
