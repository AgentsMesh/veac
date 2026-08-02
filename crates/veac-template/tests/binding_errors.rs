#![allow(clippy::duplicate_mod)]

mod support;

use support::*;
use veac_ir::*;
use veac_template::{MediaBinding, TemplateErrorKind, TextBinding};

#[test]
fn media_binding_set_must_be_exact_and_unique() {
    let project = project(FillMode::FitDuration, false);
    let material = video(six_seconds(), 1920, 1080);
    let mut value = request(&project, material.clone());
    value.media_bindings.clear();
    assert_eq!(
        error_kind(&project, &value),
        TemplateErrorKind::MissingMediaBinding
    );

    value.media_bindings.push(MediaBinding {
        clip_id: ItemId::new("itm_extra").unwrap(),
        material: material.clone(),
    });
    assert_eq!(
        error_kind(&project, &value),
        TemplateErrorKind::UnexpectedMediaBinding
    );

    value.media_bindings = vec![
        MediaBinding {
            clip_id: ItemId::new("itm_slot").unwrap(),
            material: material.clone(),
        },
        MediaBinding {
            clip_id: ItemId::new("itm_slot").unwrap(),
            material,
        },
    ];
    assert_eq!(
        error_kind(&project, &value),
        TemplateErrorKind::DuplicateMediaBinding
    );
}

#[test]
fn text_bindings_are_optional_but_typed_unique_and_nonempty() {
    let project = project(FillMode::FitDuration, true);
    let mut value = request(&project, video(six_seconds(), 1920, 1080));
    value.text_bindings.push(TextBinding {
        clip_id: ItemId::new("itm_unknown").unwrap(),
        text: "x".to_owned(),
    });
    assert_eq!(
        error_kind(&project, &value),
        TemplateErrorKind::UnexpectedTextBinding
    );

    value.text_bindings = vec![text_binding("a"), text_binding("b")];
    assert_eq!(
        error_kind(&project, &value),
        TemplateErrorKind::DuplicateTextBinding
    );

    value.text_bindings = vec![text_binding("")];
    assert_eq!(error_kind(&project, &value), TemplateErrorKind::InvalidText);
}

#[test]
fn display_diagnostics_name_the_error_kind_and_clip() {
    let project = project(FillMode::FitDuration, false);
    let mut value = request(&project, video(six_seconds(), 1920, 1080));
    value.media_bindings.clear();
    let error = veac_template::propose_template_fill(&project, &value).unwrap_err();
    let rendered = error.to_string();
    assert!(rendered.contains("MissingMediaBinding"));
    assert!(rendered.contains("itm_slot"));
}
