use super::super::{
    plugin_effects, PluginParameterTypeSpec, StandardLibrarySpec, VocabularyValidationError,
};

#[test]
fn plugin_validation_rejects_each_identity_precondition() {
    assert_mutation("identity is not canonical", |plugin| plugin.schema.clear());
    assert_mutation("identity is not canonical", |plugin| {
        plugin.schema_version = 0
    });
    assert_mutation("identity is not canonical", |plugin| {
        plugin.digest = "abc".into()
    });
    assert_mutation("identity is not canonical", |plugin| {
        plugin.digest = "G".repeat(64)
    });
    assert_mutation("identity is not canonical", |plugin| {
        plugin.effect_type.push_str(".changed")
    });
}

#[test]
fn plugin_validation_checks_both_standard_constructors() {
    assert_mutation("standard library", |plugin| {
        plugin.application_constructor = "unknown_application".into()
    });
}

#[test]
fn plugin_validation_rejects_duplicate_and_untyped_parameters() {
    assert_mutation("parameter contract", |plugin| {
        plugin.parameters.push(plugin.parameters[0].clone())
    });
    assert_mutation("parameter contract", |plugin| {
        plugin.parameters[0].minimum = Some("not-a-number".into())
    });
    assert_mutation("parameter contract", |plugin| {
        let parameter = &mut plugin.parameters[0];
        parameter.value_type = PluginParameterTypeSpec::Color;
        parameter.minimum = None;
        parameter.maximum = None;
        parameter.supports_curve = true;
    });
}

#[test]
fn locally_valid_non_numeric_parameters_still_fail_the_closed_catalog() {
    for value_type in [
        PluginParameterTypeSpec::Boolean,
        PluginParameterTypeSpec::Color,
    ] {
        let mut values = plugin_effects::current();
        let parameter = &mut values[0].parameters[0];
        parameter.value_type = value_type;
        parameter.minimum = None;
        parameter.maximum = None;
        parameter.supports_curve = false;
        assert_error(
            plugin_effects::validate(&values, &StandardLibrarySpec::current()),
            "closed registry",
        );
    }
}

#[test]
fn locally_valid_descriptor_metadata_cannot_extend_the_registry() {
    assert_mutation("closed registry", |plugin| {
        plugin.namespace.push_str(".extension")
    });
    assert_mutation("closed registry", |plugin| plugin.parameters.clear());
}

fn assert_mutation(expected: &str, mutation: impl FnOnce(&mut super::super::PluginEffectSpec)) {
    let mut values = plugin_effects::current();
    mutation(&mut values[0]);
    assert_error(
        plugin_effects::validate(&values, &StandardLibrarySpec::current()),
        expected,
    );
}

fn assert_error(result: Result<(), VocabularyValidationError>, expected: &str) {
    let error = result.unwrap_err().to_string();
    assert!(error.contains(expected), "unexpected error: {error}");
}
