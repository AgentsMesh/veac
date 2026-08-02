use crate::authoring::{format_document, lower_document, parse};

use super::project;

fn resource_error(resource: &str) -> crate::authoring::Diagnostics {
    parse(&project(&format!("{resource}\nsequence main {{}}"))).unwrap_err()
}

#[test]
fn font_remote_and_default_media_streams_lower_to_exact_material_contracts() {
    let source = project(
        r#"resource video picture {
  locator local { path "picture.mov"; }
  streams { video auto; audio disabled; }
}
resource audio voice {
  locator remote {
    uri "https://cdn.example.test/voice.wav";
    identity sha256 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
  }
  streams { video disabled; audio auto; }
}
resource font inter { locator local { path "Inter.ttf"; } }
sequence main {}"#,
    );
    let mut document = parse(&source).unwrap_or_else(|error| panic!("{error}"));
    let formatted = format_document(&document);
    assert_eq!(format_document(&parse(&formatted).unwrap()), formatted);
    for resource in &mut document.project.resources {
        if matches!(
            resource.kind,
            crate::authoring::ResourceKind::Video | crate::authoring::ResourceKind::Audio
        ) {
            resource.streams = None;
        }
    }
    let envelope = lower_document(&document).unwrap_or_else(|error| panic!("{error}"));
    let materials = &envelope.project.materials;
    assert_eq!(materials.len(), 3);
    let video = materials
        .iter()
        .find(|value| value.id.as_str() == "med_picture")
        .unwrap();
    assert_eq!(video.kind, veac_ir::MaterialKind::Video);
    assert_eq!(video.stream_intent.video, veac_ir::StreamChoice::Auto);
    assert_eq!(video.stream_intent.audio, veac_ir::StreamChoice::Auto);
    let audio = materials
        .iter()
        .find(|value| value.id.as_str() == "med_voice")
        .unwrap();
    assert_eq!(audio.kind, veac_ir::MaterialKind::Audio);
    assert_eq!(audio.stream_intent.video, veac_ir::StreamChoice::Disabled);
    assert_eq!(audio.stream_intent.audio, veac_ir::StreamChoice::Auto);
    assert_eq!(
        audio.source,
        veac_ir::MaterialSource::Remote {
            uri: "https://cdn.example.test/voice.wav".to_owned()
        }
    );
    assert_eq!(
        audio.identity.as_ref().unwrap().algorithm,
        veac_ir::HashAlgorithm::Sha256
    );
    assert_eq!(audio.identity.as_ref().unwrap().digest, "a".repeat(64));
    let font = materials
        .iter()
        .find(|value| value.id.as_str() == "med_inter")
        .unwrap();
    assert_eq!(font.kind, veac_ir::MaterialKind::Font);
    assert_eq!(
        font.source,
        veac_ir::MaterialSource::File {
            uri: "Inter.ttf".to_owned()
        }
    );
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn local_and_remote_locator_fields_report_missing_duplicate_and_bad_shapes() {
    for (resource, code) in [
        (
            "resource image x { locator local {} }",
            "AUTHORING_REQUIRED_FIELD",
        ),
        (
            "resource image x { locator local { path one two; } }",
            "AUTHORING_LOCATOR_FIELD",
        ),
        (
            "resource image x { locator local { path \"a\"; path \"b\"; } }",
            "AUTHORING_DUPLICATE_FIELD",
        ),
        (
            "resource image x { locator remote { identity sha256 \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"; } }",
            "AUTHORING_REQUIRED_FIELD",
        ),
        (
            "resource image x { locator remote { uri \"https://x/y.png\"; } }",
            "AUTHORING_REQUIRED_FIELD",
        ),
        (
            "resource image x { locator remote { uri \"https://x/y.png\"; identity md5 \"abc\"; } }",
            "AUTHORING_RESOURCE_IDENTITY",
        ),
        (
            "resource image x { locator remote { uri \"https://x/y.png\"; identity sha256 \"a\" \"b\"; } }",
            "AUTHORING_RESOURCE_IDENTITY",
        ),
    ] {
        let error = resource_error(resource);
        assert!(
            error.as_slice().iter().any(|value| value.code == code),
            "missing {code}: {error}"
        );
    }
}

#[test]
fn source_parser_rejects_unknown_types_and_wrong_reference_kinds() {
    for (authored, code) in [
        ("source mystery value;", "AUTHORING_SOURCE_TYPE"),
        ("source media sequence main;", "AUTHORING_REFERENCE_KIND"),
        (
            "source sequence resource media;",
            "AUTHORING_REFERENCE_KIND",
        ),
    ] {
        let source = project(&format!(
            r#"sequence main {{
  layer visual picture {{
    item broken {{ {authored} record {{ at 0s; duration 1s; }} }}
  }}
}}"#
        ));
        let error = parse(&source).unwrap_err();
        assert!(error.as_slice().iter().any(|value| value.code == code));
    }
}
