#![allow(clippy::duplicate_mod)]

mod support;

use support::*;
use veac_ir::*;
use veac_template::propose_template_fill;

#[test]
fn clip_frame_units_are_resolved_against_the_sequence_canvas() {
    for (unit, width, height) in [
        (LengthUnit::Pixels, 400.0, 400.0),
        (LengthUnit::Normalized, 0.5, 0.5),
        (LengthUnit::Percent, 50.0, 50.0),
    ] {
        let mut project = project(FillMode::FitDuration, false);
        project.project.sequences[0].tracks[0].clips[0]
            .visual
            .as_mut()
            .unwrap()
            .frame = Some(Frame {
            width: Length { value: width, unit },
            height: Length {
                value: height,
                unit,
            },
            fit: veac_ir::FitMode::Cover,
        });
        validate(&project).unwrap();
        let request = request(&project, video(six_seconds(), 1920, 1080));
        let output = apply(
            &project,
            &propose_template_fill(&project, &request).unwrap(),
        );
        assert!(clip(&output, "itm_slot")
            .visual
            .as_ref()
            .unwrap()
            .transform
            .crop
            .is_some());
    }
}

#[test]
fn sample_aspect_ratio_and_rotation_affect_display_crop() {
    let project = project(FillMode::FitDuration, false);
    let mut replacement = video(six_seconds(), 1920, 1080);
    let info = replacement.probe.as_mut().unwrap().streams[0]
        .video
        .as_mut()
        .unwrap();
    info.sample_aspect_ratio = Rational::new(2, 1).unwrap();
    info.rotation_degrees = 90;
    let request = request(&project, replacement);
    let output = apply(
        &project,
        &propose_template_fill(&project, &request).unwrap(),
    );
    let crop = clip(&output, "itm_slot")
        .visual
        .as_ref()
        .unwrap()
        .transform
        .crop
        .as_ref()
        .unwrap();
    let Animatable::Constant { value: crop } = crop else {
        panic!("template crop must be static")
    };
    assert_eq!(crop.width, 1.0);
    assert_eq!(crop.height, 0.5);
    assert_eq!(crop.y, 0.25);
}

#[test]
fn arbitrary_rotation_uses_the_expanded_display_bounds() {
    let project = project(FillMode::FitDuration, false);
    let mut replacement = video(six_seconds(), 1920, 1080);
    replacement.probe.as_mut().unwrap().streams[0]
        .video
        .as_mut()
        .unwrap()
        .rotation_degrees = 45;
    let request = request(&project, replacement);
    let output = apply(
        &project,
        &propose_template_fill(&project, &request).unwrap(),
    );
    let crop = clip(&output, "itm_slot")
        .visual
        .as_ref()
        .unwrap()
        .transform
        .crop
        .as_ref()
        .unwrap();
    let Animatable::Constant { value: crop } = crop else {
        panic!("template crop must be static")
    };
    assert!((crop.width - 0.5625).abs() < 1e-12);
    assert!((crop.x - 0.21875).abs() < 1e-12);
}

#[test]
fn replaceable_media_without_authored_visuals_is_invalid() {
    let mut project = project(FillMode::FitDuration, false);
    project.project.sequences[0].tracks[0].clips[0].visual = None;
    let request = request(&project, video(six_seconds(), 1920, 1080));
    assert_eq!(
        error_kind(&project, &request),
        veac_template::TemplateErrorKind::InvalidProject
    );
}
