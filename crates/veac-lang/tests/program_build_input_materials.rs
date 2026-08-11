use veac_lang::program::{
    prepare_source, BuildInputBinding, BuildInputManifestV1, BuildInputManifestValue,
    MaterialInputAuthority, MaterialInputKind, SourceIndexBuildInputType,
};

#[path = "program_functions/support.rs"]
mod support;

const DECLARATION: &str = r#"
struct MaterialBinding {
  kind: text,
  path: text,
  sha256: text,
  authority: text,
  artifact_key: text,
  video_stream: int,
  audio_stream: int,
}
input material picture: MaterialBinding;
fn material_duration() -> time {
  let selected_path = picture.path;
  1s
}
"#;

fn source() -> String {
    support::project_with(DECLARATION, "material_duration()")
}

fn input(authority: MaterialInputAuthority) -> BuildInputManifestV1 {
    let mut manifest = BuildInputManifestV1::empty();
    manifest.inputs.push(BuildInputBinding {
        name: "picture".into(),
        value: BuildInputManifestValue::Material {
            kind: MaterialInputKind::Image,
            path: "stills/cover.png".into(),
            sha256: "a".repeat(64),
            authority,
            video_stream: None,
            audio_stream: None,
        },
    });
    manifest
}

#[test]
fn material_binding_is_typed_indexed_and_identity_sensitive() {
    let prepared = prepare_source(&source()).unwrap();
    let declaration = &prepared.build_input_declarations()["picture"];
    assert_eq!(declaration.role().as_str(), "material");
    let indexed = &prepared.source_inventory().unwrap().build_inputs[0].value_type;
    assert!(
        matches!(indexed, SourceIndexBuildInputType::Material { name, .. }
        if name == "MaterialBinding")
    );

    let local = prepared
        .execute_with_inputs(&input(MaterialInputAuthority::ProjectMaterial))
        .unwrap();
    let artifact = prepared
        .execute_with_inputs(&input(MaterialInputAuthority::Artifact {
            artifact_key: "b".repeat(64),
        }))
        .unwrap();
    assert_ne!(
        local.envelope().executable.digests.declared_inputs_sha256,
        artifact
            .envelope()
            .executable
            .digests
            .declared_inputs_sha256
    );
}

#[test]
fn material_role_and_nominal_shape_are_closed() {
    for declaration in [
        "input material picture: text;",
        "struct Other { kind: text, path: text, sha256: text, authority: text, artifact_key: text, video_stream: int, audio_stream: int, } input material picture: Other;",
        "struct MaterialBinding { path: text, kind: text, sha256: text, authority: text, artifact_key: text, video_stream: int, audio_stream: int, } input material picture: MaterialBinding;",
        "struct MaterialBinding { kind: text, path: text, sha256: text, authority: text, artifact_key: text, video_stream: int, audio_stream: int, } input parameter picture: MaterialBinding;",
    ] {
        let source = support::project_with(declaration, "1s");
        assert_eq!(
            prepare_source(&source).unwrap_err().as_slice()[0].code,
            "PROGRAM_INPUT_TYPE"
        );
    }
}

#[test]
fn material_identity_and_path_validation_fail_closed() {
    let prepared = prepare_source(&source()).unwrap();
    for (path, sha256, authority) in [
        (
            "../cover.png",
            "a".repeat(64),
            MaterialInputAuthority::ProjectMaterial,
        ),
        (
            "cover.png",
            "A".repeat(64),
            MaterialInputAuthority::ProjectMaterial,
        ),
        (
            "cover.png",
            "a".repeat(64),
            MaterialInputAuthority::Artifact {
                artifact_key: "short".into(),
            },
        ),
    ] {
        let mut manifest = input(authority);
        let BuildInputManifestValue::Material {
            path: value_path,
            sha256: value_sha256,
            ..
        } = &mut manifest.inputs[0].value
        else {
            unreachable!()
        };
        *value_path = path.into();
        *value_sha256 = sha256;
        assert_eq!(
            prepared
                .execute_with_inputs(&manifest)
                .unwrap_err()
                .as_slice()[0]
                .code,
            "PROGRAM_INPUT_MATERIAL"
        );
    }
}

#[test]
fn material_manifest_schema_publishes_the_closed_authority_union() {
    let schema = veac_lang::program::build_input_manifest_json_schema()
        .unwrap()
        .to_string();
    assert!(schema.contains("project_material"));
    assert!(schema.contains("artifact_key"));
    assert!(schema.contains("material"));
}

#[test]
fn material_manifest_rejects_duplicate_nested_authority_keys() {
    let json = format!(
        r#"{{"schema":"https://veac.dev/schemas/build-inputs","schema_version":1,"inputs":[{{"name":"picture","value":{{"type":"material","kind":"image","path":"cover.png","sha256":"{}","authority":{{"type":"artifact","artifact_key":"{}","artifact_key":"{}"}},"video_stream":null,"audio_stream":null}}}}]}}"#,
        "a".repeat(64),
        "b".repeat(64),
        "c".repeat(64)
    );
    assert_eq!(
        veac_lang::program::parse_build_input_manifest(&json)
            .unwrap_err()
            .code(),
        "PROGRAM_INPUT_MANIFEST_JSON"
    );
}
