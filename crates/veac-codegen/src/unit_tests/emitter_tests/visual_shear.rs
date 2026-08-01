use veac_codegen::emitter::{emit_all, CodegenErrorKind};
use veac_plan::canonical::{Animatable, Vec2, MAX_VISUAL_SHEAR};

use super::support::{bindings, emit_video_command, fixture, resolved};

#[test]
fn shear_uses_alpha_precision_pivot_padding_and_bilinear_filtering() {
    let mut plan = resolved(&fixture());
    let visual = plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap();
    visual.transform.anchor = Vec2 { x: 1.0, y: 0.0 };
    visual.transform.shear = Vec2 { x: 0.75, y: -0.5 };
    let graph = graph(&plan);

    assert!(
        graph.contains("format=gbrap16le[shearprecisionv"),
        "{graph}"
    );
    assert!(graph.contains("pad=w='2*max(1*iw\\,(1-1)*iw)'"), "{graph}");
    assert!(graph.contains("pad=w='ceil(iw+0.75*ih)'"), "{graph}");
    assert!(graph.contains("h='ceil(ih+0.5*iw)'"), "{graph}");
    assert!(
        graph.contains("shear=shx=0.75:shy=-0.5:fillcolor=black@0:interp=bilinear"),
        "{graph}"
    );
}

#[test]
fn identity_shear_is_elided_and_invalid_factors_fail_preflight() {
    assert!(!graph(&resolved(&fixture())).contains("shear="));

    for shear in [
        Vec2 {
            x: MAX_VISUAL_SHEAR + 0.01,
            y: 0.0,
        },
        Vec2 {
            x: 0.0,
            y: f64::NAN,
        },
    ] {
        let mut plan = resolved(&fixture());
        plan.sequences[0].tracks[0].clips[0]
            .visual
            .as_mut()
            .unwrap()
            .transform
            .shear = shear;
        let error = emit_all(&plan, &bindings(&plan)).unwrap_err();
        let diagnostic = error.diagnostics().first().unwrap();
        assert_eq!(diagnostic.kind, CodegenErrorKind::InvalidPlan);
        assert_eq!(diagnostic.code, "PLAN_VISUAL_INVALID");
        assert!(diagnostic.message.contains("[-2, 2]"));
    }
}

#[test]
fn shear_pivot_padding_is_charged_to_the_visual_budget() {
    let mut plan = resolved(&fixture());
    let transform = &mut plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .transform;
    transform.scale = Animatable::constant(Vec2 { x: 4.1, y: 4.1 });
    transform.shear = Vec2 { x: 0.1, y: 0.0 };

    let error = emit_all(&plan, &bindings(&plan)).unwrap_err();
    assert_eq!(
        error.diagnostics().first().unwrap().code,
        "PLAN_BUDGET_VISUAL_INTERMEDIATE_PIXELS"
    );
}

fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}
