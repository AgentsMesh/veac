#![allow(clippy::duplicate_mod)]

mod support;

use support::*;
use veac_ir::*;
use veac_template::propose_template_fill;

#[test]
fn fit_duration_consumes_the_complete_video_exactly() {
    let project = project(FillMode::FitDuration, false);
    let batch = propose_template_fill(
        &project,
        &request(&project, video(six_seconds(), 1920, 1080)),
    )
    .unwrap();
    assert!(batch.atomic);
    assert!(batch.operations.iter().any(|operation| matches!(
        operation,
        EditOperation::EditStructure {
            edit: StructureEdit::SetMaterial { .. }
        }
    )));
    let output = apply(&project, &batch);
    let (start, rate) = linear(clip(&output, "itm_slot"));
    assert_eq!(start, time(0));
    assert_eq!(rate, Rational::new(3, 2).unwrap());
}

#[test]
fn take_head_uses_unit_rate_and_source_zero() {
    let project = project(FillMode::TakeHead, false);
    let batch = propose_template_fill(
        &project,
        &request(&project, video(six_seconds(), 1920, 1080)),
    )
    .unwrap();
    let output = apply(&project, &batch);
    assert_eq!(
        linear(clip(&output, "itm_slot")),
        (time(0), Rational::new(1, 1).unwrap())
    );
}

#[test]
fn take_center_uses_the_exact_middle_at_unit_rate() {
    let project = project(FillMode::TakeCenter, false);
    let batch = propose_template_fill(
        &project,
        &request(&project, video(six_seconds(), 1920, 1080)),
    )
    .unwrap();
    let output = apply(&project, &batch);
    assert_eq!(
        linear(clip(&output, "itm_slot")),
        (time(600), Rational::new(1, 1).unwrap())
    );
}

#[test]
fn image_fill_is_static_and_aspect_fills_the_canvas() {
    let project = project(FillMode::TakeCenter, false);
    let batch = propose_template_fill(&project, &request(&project, image(1000, 1000))).unwrap();
    let output = apply(&project, &batch);
    let clip = clip(&output, "itm_slot");
    assert_eq!(linear(clip), (time(0), Rational::new(1, 1).unwrap()));
    let crop = match clip
        .visual
        .as_ref()
        .unwrap()
        .transform
        .crop
        .as_ref()
        .unwrap()
    {
        veac_ir::Animatable::Constant { value } => *value,
        veac_ir::Animatable::Keyframes { .. } => panic!("template crop must be static"),
    };
    assert_eq!(crop.width, 0.5625);
    assert_eq!(crop.x, 0.21875);
    assert_eq!(output.project.materials[0].kind, MaterialKind::Image);
}
