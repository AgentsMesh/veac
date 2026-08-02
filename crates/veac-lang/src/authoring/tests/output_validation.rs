use crate::authoring::{lower_document, parse};

use super::project;

fn caption_project(tracks: &str) -> String {
    project(&format!(
        r#"
  sequence main {{
    layer caption zeta {{
      item z {{ source caption {{ content "Z"; }} record {{ at 0s; duration 1s; }} }}
    }}
    layer caption alpha {{
      item a {{ source caption {{ content "A"; }} record {{ at 0s; duration 1s; }} }}
    }}
    layer audio dialogue {{}}
  }}
  delivery captions {{
    sequence main;
    artifact caption-sidecar captions {{
      target file "captions.vtt";
      source caption-tracks {{ {tracks} }}
      encode web-vtt;
    }}
  }}
"#
    ))
}

#[test]
fn caption_tracks_sort_during_lowering_without_language_reordering() {
    let document = parse(&caption_project("track zeta; track alpha;")).unwrap();
    let envelope = lower_document(&document).unwrap();
    let veac_ir::DeliverableKind::CaptionSidecar(sidecar) =
        &envelope.project.render_configs[0].deliverables[0].kind
    else {
        panic!("caption sidecar expected")
    };
    assert_eq!(sidecar.track_ids[0].as_str(), "trk_alpha");
    assert_eq!(sidecar.track_ids[1].as_str(), "trk_zeta");
}

#[test]
fn caption_tracks_reject_duplicates_and_wrong_kinds() {
    let duplicate = parse(&caption_project("track zeta; track zeta;")).unwrap_err();
    assert!(duplicate
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_DELIVERY_CAPTION_TRACK_DUPLICATE"));
    let wrong = parse(&caption_project("track dialogue;")).unwrap_err();
    assert!(wrong
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_DELIVERY_CAPTION_TRACK_KIND"));
}

#[test]
fn file_target_is_a_bounded_safe_basename() {
    let name = "a".repeat(256);
    let invalid = caption_project("track zeta;").replace("captions.vtt", &name);
    let diagnostics = parse(&invalid).unwrap_err();
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_ARTIFACT_TARGET_FILE"));
}
