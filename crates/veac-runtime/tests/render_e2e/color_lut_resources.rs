use tempfile::tempdir;
use veac_artifact::ExecutionBindings;
use veac_codegen::emitter::{self, CodegenErrorKind};
use veac_plan::resolve_one;

use super::support::*;

#[test]
fn one_and_three_dimensional_cube_luts_execute_in_real_ffmpeg() {
    let temp = tempdir().unwrap();
    let source_color = color(70, 120, 180);
    let baseline = temp.path().join("identity-baseline.mp4");
    render_color(source_color, None, None, None, &baseline);
    let expected = rgb_at(&baseline, 0.5, WIDTH / 2, HEIGHT / 2);
    for (kind, interpolation, name) in [
        (MaterialKind::Lut1d, LutInterpolation::Linear, "one"),
        (MaterialKind::Lut3d, LutInterpolation::Tetrahedral, "three"),
    ] {
        let lut = temp.path().join(format!("identity {name}:final,look.cube"));
        write_identity_lut(&lut, kind);
        let output = temp.path().join(format!("identity-{name}.mp4"));
        render_color(
            source_color,
            Some(lut_pipeline(interpolation)),
            Some((kind, &lut)),
            None,
            &output,
        );
        let actual = rgb_at(&output, 0.5, WIDTH / 2, HEIGHT / 2);
        assert!(
            actual
                .into_iter()
                .zip(expected)
                .all(|(actual, expected)| actual.abs_diff(expected) <= 8),
            "{kind:?}: expected={expected:?}, actual={actual:?}"
        );
    }
}

#[test]
fn malformed_cube_fails_before_a_backend_bundle_can_spawn() {
    let temp = tempdir().unwrap();
    let lut = temp.path().join("malformed.cube");
    std::fs::write(&lut, "LUT_3D_SIZE 2\n0 0 0\n").unwrap();
    let output = temp.path().join("must-not-exist.mp4");
    let mut canonical = color_project(
        color(70, 120, 180),
        Some(lut_pipeline(LutInterpolation::Tetrahedral)),
        Some(MaterialKind::Lut3d),
        None,
    );
    let assets = BTreeMap::from([("med_test_lut".to_owned(), lut)]);
    hydrate(&mut canonical, &assets);
    let output_id = canonical.project.render_configs[0].id.clone();
    let plan = resolve_one(&canonical, &output_id).unwrap();
    let paths = plan
        .inputs
        .iter()
        .map(|input| {
            let material = input.material_id.as_ref().unwrap();
            (input.id.clone(), assets[material.as_str()].clone())
        })
        .collect();
    let mut bindings = ExecutionBindings::from_originals(&plan, &paths).unwrap();
    bindings
        .bind_output(plan.output.deliverables[0].id.clone(), output.clone())
        .unwrap();
    let error = emitter::emit_all(&plan, &bindings).unwrap_err();
    assert_eq!(
        error.diagnostics()[0].kind,
        CodegenErrorKind::InvalidResourceBinding
    );
    assert_eq!(error.diagnostics()[0].code, "LUT_RESOURCE_INVALID");
    assert!(!output.exists());
}

fn lut_pipeline(interpolation: LutInterpolation) -> ColorPipeline {
    ColorPipeline {
        input: rec709(),
        working: rec709(),
        output: rec709(),
        stages: vec![ColorStage::Lut {
            application: LutApplication {
                material_id: MaterialId::new("med_test_lut").unwrap(),
                interpolation,
            },
        }],
    }
}
