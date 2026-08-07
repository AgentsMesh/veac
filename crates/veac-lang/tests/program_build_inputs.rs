use veac_lang::program::{
    prepare_source, BuildInputBinding, BuildInputManifestV1, BuildInputManifestValue as InputValue,
    BuildInputRole,
};

#[path = "program_functions/support.rs"]
mod support;

fn source(extra: &str) -> String {
    let declarations = format!(
        r#"
input parameter enabled: bool;
input asset_metadata duration: time;
input analysis tint: color;
{extra}
fn chosen_duration() -> time {{ if enabled {{ duration }} else {{ 1s }} }}
"#
    );
    support::project_with(&declarations, "chosen_duration()")
        .replace("generator_transparent()", "generator_solid(tint)")
}

fn manifest(enabled: bool, duration: &str, tint: &str) -> BuildInputManifestV1 {
    let mut value = BuildInputManifestV1::empty();
    value.inputs = vec![
        BuildInputBinding {
            name: "enabled".into(),
            value: InputValue::Bool { value: enabled },
        },
        BuildInputBinding {
            name: "duration".into(),
            value: InputValue::Time {
                value: duration.into(),
            },
        },
        BuildInputBinding {
            name: "tint".into(),
            value: InputValue::Color { value: tint.into() },
        },
    ];
    value
}

#[test]
fn declared_typed_inputs_drive_graph_build_and_stable_core_identity() {
    let prepared = prepare_source(&source("")).unwrap();
    let declarations = prepared.build_input_declarations();
    assert_eq!(declarations["enabled"].role(), BuildInputRole::Parameter);
    assert_eq!(
        declarations["duration"].role(),
        BuildInputRole::AssetMetadata
    );
    assert_eq!(declarations["tint"].role(), BuildInputRole::Analysis);
    let ids = prepared
        .entry_function()
        .body()
        .inputs()
        .iter()
        .map(|input| input.identity())
        .collect::<Vec<_>>();
    let tint = &declarations["tint"];
    assert!(ids.iter().any(|identity| {
        matches!(identity, veac_lang::program::expression::CoreInputIdentity::Build(id)
            if *id == tint.id())
    }));

    let built = prepared
        .execute_with_inputs(&manifest(true, "2500ms", "#102030ff"))
        .unwrap();
    assert_eq!(support::result_duration(&built), "2500ms");
    veac_ir::validate(built.envelope()).unwrap();
}

#[test]
fn input_binding_is_deterministic_and_value_sensitive_in_provenance() {
    let prepared = prepare_source(&source("")).unwrap();
    let first = prepared
        .execute_with_inputs(&manifest(true, "2s", "#102030ff"))
        .unwrap();
    let same = prepared
        .execute_with_inputs(&manifest(true, "2s", "#102030ff"))
        .unwrap();
    let changed = prepared
        .execute_with_inputs(&manifest(false, "2s", "#102030ff"))
        .unwrap();
    let equivalent = prepared
        .execute_with_inputs(&manifest(true, "2000ms", "#102030FF"))
        .unwrap();
    assert_eq!(
        veac_ir::canonical_json(first.envelope()).unwrap(),
        veac_ir::canonical_json(same.envelope()).unwrap()
    );
    assert_eq!(
        first.envelope().executable.digests.declared_inputs_sha256,
        equivalent
            .envelope()
            .executable
            .digests
            .declared_inputs_sha256
    );
    assert_ne!(
        first.envelope().executable.digests.declared_inputs_sha256,
        changed.envelope().executable.digests.declared_inputs_sha256
    );
}

#[test]
fn binding_failures_are_closed_before_publication() {
    let prepared = prepare_source(&source("")).unwrap();
    assert_eq!(
        prepared.execute().unwrap_err().as_slice()[0].code,
        "PROGRAM_INPUT_MISSING"
    );

    let mut unknown = manifest(true, "2s", "#102030ff");
    unknown.inputs.push(BuildInputBinding {
        name: "ambient".into(),
        value: InputValue::Text {
            value: "forbidden".into(),
        },
    });
    assert_error(&prepared, &unknown, "PROGRAM_INPUT_UNKNOWN");

    let mut duplicate = manifest(true, "2s", "#102030ff");
    duplicate.inputs.push(duplicate.inputs[0].clone());
    assert_error(&prepared, &duplicate, "PROGRAM_INPUT_DUPLICATE");

    let mut mismatch = manifest(true, "2s", "#102030ff");
    mismatch.inputs[1].value = InputValue::Scalar {
        value: "2.0".into(),
    };
    assert_error(&prepared, &mismatch, "PROGRAM_INPUT_TYPE_MISMATCH");

    let mut invalid = manifest(true, "NaNs", "#102030ff");
    assert_error(&prepared, &invalid, "PROGRAM_INPUT_VALUE");
    invalid.inputs[1].value = InputValue::Time {
        value: "2px".into(),
    };
    assert_error(&prepared, &invalid, "PROGRAM_INPUT_VALUE");
}

#[test]
fn declarations_are_root_only_unique_closed_leaf_types() {
    for (source, code) in [
        (
            "module { input parameter title: text; }",
            "PROGRAM_INPUT_ENTRY_ONLY",
        ),
        (
            "input parameter x: text; input analysis x: text; fn main(context: Context) -> Project { context }",
            "PROGRAM_DUPLICATE_SYMBOL",
        ),
        (
            "input parameter values: list<text>; fn main(context: Context) -> Project { context }",
            "PROGRAM_INPUT_TYPE",
        ),
    ] {
        assert_eq!(prepare_source(source).unwrap_err().as_slice()[0].code, code);
    }
}

fn assert_error(
    prepared: &veac_lang::program::ExecutableBuild,
    manifest: &BuildInputManifestV1,
    code: &str,
) {
    assert_eq!(
        prepared
            .execute_with_inputs(manifest)
            .unwrap_err()
            .as_slice()[0]
            .code,
        code
    );
}
