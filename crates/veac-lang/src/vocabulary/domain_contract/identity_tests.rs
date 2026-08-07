use super::{
    identity, DomainEffectSpec, DomainOpsetSpec, DomainRuntimeActionSpec, DomainValueShapeSpec,
};

#[test]
fn published_preimage_recomputes_the_runtime_registry_identity() {
    let value = DomainOpsetSpec::current();
    assert_eq!(identity::calculate(&value).unwrap(), value.registry_digest);

    let mut changed = value;
    changed.operations[0].contract.effect = DomainEffectSpec::LocalMutation;
    assert_ne!(
        identity::calculate(&changed).unwrap(),
        changed.registry_digest
    );
}

#[test]
fn published_runtime_action_is_part_of_the_registry_identity() {
    let mut value = DomainOpsetSpec::current();
    let original = value.registry_digest.clone();
    value.operations[0].contract.runtime_action = DomainRuntimeActionSpec::EntityConstructor;
    assert_ne!(identity::calculate(&value).unwrap(), original);
}

#[test]
fn published_surface_is_part_of_the_registry_identity() {
    let value = DomainOpsetSpec::current();
    let mut library = super::super::StandardLibrarySpec::current();
    let plugins = super::super::plugin_effects::current();
    let original = identity::calculate_with_plugins(&value, &library, &plugins).unwrap();
    library.free_functions[0].name.push_str("_changed");
    assert_ne!(
        identity::calculate_with_plugins(&value, &library, &plugins).unwrap(),
        original
    );
}

#[test]
fn published_plugin_effect_identity_is_part_of_the_registry_digest() {
    let value = DomainOpsetSpec::current();
    let library = super::super::StandardLibrarySpec::current();
    let mut plugins = super::super::plugin_effects::current();
    let original = identity::calculate_with_plugins(&value, &library, &plugins).unwrap();
    let mut missing = plugins.clone();
    missing.clear();
    assert_ne!(
        identity::calculate_with_plugins(&value, &library, &missing).unwrap(),
        original
    );
    plugins[0].effect_type.push_str(".changed");
    assert_ne!(
        identity::calculate_with_plugins(&value, &library, &plugins).unwrap(),
        original
    );
    plugins = super::super::plugin_effects::current();
    plugins[0].digest = "00".repeat(32);
    assert_ne!(
        identity::calculate_with_plugins(&value, &library, &plugins).unwrap(),
        original
    );
}

#[test]
fn identity_rejects_an_unknown_primitive_shape() {
    let mut value = DomainOpsetSpec::current();
    value.operations[0].contract.ordered_operands[0].shape = DomainValueShapeSpec::Primitive {
        name: "unknown".into(),
    };
    assert!(identity::calculate(&value)
        .unwrap_err()
        .to_string()
        .contains("unknown primitive"));
}

#[test]
fn identity_rejects_missing_duplicate_and_unknown_exposures() {
    let value = DomainOpsetSpec::current();
    let plugins = super::super::plugin_effects::current();

    let mut missing = super::super::StandardLibrarySpec::current();
    missing.free_functions.pop();
    assert_identity_error(&value, &missing, &plugins, "exposure count");

    let mut duplicate = super::super::StandardLibrarySpec::current();
    duplicate.free_functions[1].operation_opcode = duplicate.free_functions[0].operation_opcode;
    assert_identity_error(&value, &duplicate, &plugins, "more than one");

    let mut unknown = super::super::StandardLibrarySpec::current();
    unknown.free_functions[0].operation_opcode = u16::MAX;
    assert_identity_error(
        &value,
        &unknown,
        &plugins,
        "has no standard-library exposure",
    );
}

#[test]
fn identity_rejects_unknown_primitive_lists_and_results() {
    let mut list = DomainOpsetSpec::current();
    list.operations[0].contract.ordered_operands[0].shape = DomainValueShapeSpec::PrimitiveList {
        name: "unknown".into(),
    };
    assert!(identity::calculate(&list).is_err());

    let mut result = DomainOpsetSpec::current();
    result.operations[0].contract.result = DomainValueShapeSpec::Primitive {
        name: "unknown".into(),
    };
    assert!(identity::calculate(&result).is_err());
}

#[test]
fn validation_distinguishes_recomputed_and_runtime_registry_identity() {
    let mut value = DomainOpsetSpec::current();
    let library = super::super::StandardLibrarySpec::current();
    let mut plugins = super::super::plugin_effects::current();
    plugins[0].effect_type.push_str(".changed");
    value.registry_digest = identity::calculate_with_plugins(&value, &library, &plugins).unwrap();

    let error = value
        .validate_with_plugins(&plugins)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("identity does not match"),
        "unexpected error: {error}"
    );
}

fn assert_identity_error(
    value: &DomainOpsetSpec,
    library: &super::super::StandardLibrarySpec,
    plugins: &[super::super::PluginEffectSpec],
    expected: &str,
) {
    let error = identity::calculate_with_plugins(value, library, plugins)
        .unwrap_err()
        .to_string();
    assert!(error.contains(expected), "unexpected error: {error}");
}
