use veac_lang::program::{
    prepare_source, BuildInputBinding, BuildInputManifestV1, BuildInputManifestValue as Value,
};

#[path = "program_functions/support.rs"]
mod support;

fn source() -> String {
    let declarations = r#"
input parameter flag: bool;
input parameter count: int;
input parameter ratio: scalar;
input parameter label: text;
input asset_metadata duration: time;
input asset_metadata width: length;
input analysis rotation: angle;
input analysis tint: color;
fn observe() -> time {
  let a = flag;
  let b = count;
  let c = ratio;
  let d = label;
  let e = width;
  let f = rotation;
  let g = tint;
  duration
}
"#;
    support::project_with(declarations, "observe()")
}

fn binding(name: &str, value: Value) -> BuildInputBinding {
    BuildInputBinding {
        name: name.into(),
        value,
    }
}

fn manifest() -> BuildInputManifestV1 {
    let mut result = BuildInputManifestV1::empty();
    result.inputs = vec![
        binding("flag", Value::Bool { value: true }),
        binding("count", Value::Integer { value: 7 }),
        binding(
            "ratio",
            Value::Scalar {
                value: "1.25".into(),
            },
        ),
        binding(
            "label",
            Value::Text {
                value: "显式输入".into(),
            },
        ),
        binding(
            "duration",
            Value::Time {
                value: "1250ms".into(),
            },
        ),
        binding(
            "width",
            Value::Length {
                value: "12.5px".into(),
            },
        ),
        binding(
            "rotation",
            Value::Angle {
                value: "-45deg".into(),
            },
        ),
        binding(
            "tint",
            Value::Color {
                value: "#ABCDEF80".into(),
            },
        ),
    ];
    result
}

#[test]
fn every_closed_leaf_value_binds_through_verified_function_inputs() {
    let built = prepare_source(&source())
        .unwrap()
        .execute_with_inputs(&manifest())
        .unwrap();
    assert_eq!(support::result_duration(&built), "1250ms");
}

#[test]
fn exact_overflow_and_wrong_units_fail_closed() {
    let prepared = prepare_source(&source()).unwrap();
    for value in [
        "999999999999999999999999999999999999999999999.0",
        "infinity",
    ] {
        let mut invalid = manifest();
        invalid.inputs[2].value = Value::Scalar {
            value: value.into(),
        };
        assert_eq!(
            prepared
                .execute_with_inputs(&invalid)
                .unwrap_err()
                .as_slice()[0]
                .code,
            "PROGRAM_INPUT_VALUE"
        );
    }

    let mut oversized = manifest();
    oversized.inputs[3].value = Value::Text {
        value: "x".repeat(1024 * 1024 + 1),
    };
    assert_eq!(
        prepared
            .execute_with_inputs(&oversized)
            .unwrap_err()
            .as_slice()[0]
            .code,
        "PROGRAM_INPUT_VALUE"
    );
}
