use tempfile::{tempdir, TempDir};
use veac_lang::program::{BuildInputManifestValue as Value, ExecutableBuild};

use super::support::{source_file, EXECUTABLE_SOURCE};

fn prepared(temp: &TempDir, declarations: &str) -> (std::path::PathBuf, ExecutableBuild) {
    let source = source_file(temp, &format!("{declarations}\n{EXECUTABLE_SOURCE}"));
    let program = veac_lang::program::prepare_path(&source).unwrap();
    (source, program)
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn validate(
    source: &std::path::Path,
    program: &ExecutableBuild,
    manifest: Option<&std::path::Path>,
    inline: &[&str],
) -> crate::CliResult<veac_lang::program::BuildInputManifestV1> {
    let manifest = crate::frontend::build_inputs(manifest, &strings(inline), program)?;
    program
        .validate_inputs(&manifest)
        .map_err(|errors| crate::diagnostic::program(source, errors))?;
    Ok(manifest)
}

#[test]
fn inline_values_decode_from_declared_types_and_preserve_text_equals() {
    let temp = tempdir().unwrap();
    let declarations = r#"
enum Locale { ZhCn, EnUs, }
input parameter angle_value: angle;
input parameter color_value: color;
input parameter count: int;
input parameter enabled: bool;
input parameter gain: scalar;
input parameter height: length;
input parameter locale: Locale;
input parameter title: text;
input parameter window: time;"#;
    let (source, program) = prepared(&temp, declarations);
    let manifest = validate(
        &source,
        &program,
        None,
        &[
            "window=250ms",
            "title=part=one=two",
            "locale=ZhCn",
            "height=42px",
            "gain=1.25",
            "enabled=true",
            "count=-7",
            "color_value=#AABBCCDD",
            "angle_value=90deg",
        ],
    )
    .unwrap();
    assert!(manifest.inputs.iter().any(|binding| {
        binding.name == "title"
            && binding.value
                == Value::Text {
                    value: "part=one=two".into(),
                }
    }));
    assert!(manifest.inputs.iter().any(|binding| {
        binding.name == "locale"
            && binding.value
                == Value::Enum {
                    value: "ZhCn".into(),
                }
    }));
}

#[test]
fn manifest_base_inline_override_and_order_share_one_typed_digest() {
    let temp = tempdir().unwrap();
    let (source, program) = prepared(
        &temp,
        "input parameter duration: time;\ninput parameter title: text;",
    );
    let path = temp.path().join("inputs.json");
    std::fs::write(
        &path,
        r#"{"schema":"https://veac.dev/schemas/build-inputs","schema_version":1,"inputs":[{"name":"duration","value":{"type":"time","value":"1s"}},{"name":"title","value":{"type":"text","value":"a=b"}}]}"#,
    )
    .unwrap();
    let mixed = validate(&source, &program, Some(&path), &["duration=2s"]).unwrap();
    let inline = validate(&source, &program, None, &["title=a=b", "duration=2000ms"]).unwrap();
    let first = program.execute_with_inputs(&mixed).unwrap();
    let second = program.execute_with_inputs(&inline).unwrap();
    assert_eq!(
        first.envelope().executable.digests.declared_inputs_sha256,
        second.envelope().executable.digests.declared_inputs_sha256
    );
}

#[test]
fn inline_contract_fails_closed() {
    let temp = tempdir().unwrap();
    let (source, program) = prepared(
        &temp,
        "input parameter count: int;\ninput parameter enabled: bool;",
    );
    for (inline, code) in [
        (vec!["count"], "PROGRAM_INPUT_INLINE_SYNTAX"),
        (vec!["=7"], "PROGRAM_INPUT_INLINE_SYNTAX"),
        (vec!["count=1", "count=2"], "PROGRAM_INPUT_DUPLICATE"),
        (vec!["unknown=1"], "PROGRAM_INPUT_UNKNOWN"),
        (vec!["count=nope", "enabled=true"], "PROGRAM_INPUT_VALUE"),
        (vec!["count=1", "enabled=yes"], "PROGRAM_INPUT_VALUE"),
        (vec!["count=1"], "PROGRAM_INPUT_MISSING"),
    ] {
        let error = validate(&source, &program, None, &inline).unwrap_err();
        assert!(error.to_string().contains(code), "{inline:?}: {error}");
    }
}

#[test]
fn overridden_duplicate_manifest_is_still_rejected() {
    let temp = tempdir().unwrap();
    let (source, program) = prepared(&temp, "input parameter count: int;");
    let path = temp.path().join("inputs.json");
    std::fs::write(
        &path,
        r#"{"schema":"https://veac.dev/schemas/build-inputs","schema_version":1,"inputs":[{"name":"count","value":{"type":"int","value":1}},{"name":"count","value":{"type":"int","value":2}}]}"#,
    )
    .unwrap();
    let error = validate(&source, &program, Some(&path), &["count=3"]).unwrap_err();
    assert!(error.to_string().contains("PROGRAM_INPUT_DUPLICATE"));
}

#[test]
fn invalid_unit_and_enum_variant_reach_the_typed_binder() {
    let temp = tempdir().unwrap();
    let declarations =
        "enum Locale { ZhCn, }\ninput parameter locale: Locale;\ninput parameter window: time;";
    let (source, program) = prepared(&temp, declarations);
    for (inline, code) in [
        (vec!["locale=ZhCn", "window=12px"], "PROGRAM_INPUT_VALUE"),
        (
            vec!["locale=Missing", "window=1s"],
            "PROGRAM_INPUT_ENUM_VARIANT",
        ),
    ] {
        let error = validate(&source, &program, None, &inline).unwrap_err();
        assert!(error.to_string().contains(code), "{error}");
    }
}

#[test]
fn material_input_cannot_be_flattened_into_an_inline_string() {
    let temp = tempdir().unwrap();
    let declaration = r#"
struct MaterialBinding {
  kind: text,
  path: text,
  sha256: text,
  authority: text,
  artifact_key: text,
  video_stream: int,
  audio_stream: int,
}
input material portrait: MaterialBinding;"#;
    let (source, program) = prepared(&temp, declaration);
    let error = validate(&source, &program, None, &["portrait=stills/hero.png"]).unwrap_err();
    assert!(error.to_string().contains("PROGRAM_INPUT_INLINE_MATERIAL"));
}
