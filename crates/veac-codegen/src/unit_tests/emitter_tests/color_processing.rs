use veac_codegen::emitter::CodegenErrorKind;
use veac_plan::canonical::*;
use veac_plan::ResolvedColorStage;

use super::support::{
    bindings, bindings_with_original, emit_video_command, graded_project, lut_fixture, resolved,
    tone_curve,
};

#[test]
fn ordered_color_pipeline_emits_metadata_adjustments_curves_wheels_and_escaped_lut() {
    let project = graded_project(MaterialKind::Lut3d, LutInterpolation::Tetrahedral);
    let plan = resolved(&project);
    let resource = plan
        .inputs
        .iter()
        .find(|input| matches!(input.kind, veac_plan::ResolvedInputKind::Resource { .. }))
        .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("look:hero,final cube.cube");
    std::fs::copy(lut_fixture(MaterialKind::Lut3d), &path).unwrap();
    let local = bindings_with_original(&plan, &resource.id, path);
    let command = emit_video_command(&plan, &local).unwrap();
    assert_eq!(
        command.inputs.len(),
        1,
        "LUT resource must not become -i media"
    );
    let graph = command.filter_graph.unwrap();
    for marker in [
        "zscale=matrixin=bt709:primariesin=bt709:transferin=bt709:rangein=tv",
        "lutrgb=r='clip(val*1.414213562373",
        "colortemperature=temperature=6000",
        "colorbalance=gm=0.1:pl=true",
        "curves=master='0/0.015",
        "huesaturation=hue=5:saturation=0.2:intensity=-0.1:colors=r",
        "curves=master='0/0 0.5/0.55 1/1':interp=pchip",
        "colorbalance=rs=0.05:gs=0:bs=-0.05",
        "look\\\\:hero\\,final cube.cube:interp=tetrahedral",
        "setparams=range=limited:color_primaries=bt709:color_trc=bt709:colorspace=bt709",
    ] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
}

#[test]
fn one_dimensional_lut_uses_resource_filter_without_media_input() {
    let project = graded_project(MaterialKind::Lut1d, LutInterpolation::Spline);
    let plan = resolved(&project);
    let command = emit_video_command(&plan, &bindings(&plan)).unwrap();
    assert_eq!(command.inputs.len(), 1);
    assert!(command.filter_graph.unwrap().contains("lut1d=file="));
}

#[test]
fn mixed_curve_interpolation_emits_master_first_and_groups_independent_channels() {
    let mut mixed = graded_project(MaterialKind::Lut3d, LutInterpolation::Tetrahedral);
    let pipeline = mixed.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .color_pipeline
        .as_mut()
        .unwrap();
    pipeline.stages.insert(
        2,
        ColorStage::Curves {
            curves: ColorCurves {
                luma: Some(tone_curve(ToneCurveInterpolation::Natural)),
                red: Some(tone_curve(ToneCurveInterpolation::Monotonic)),
                green: Some(tone_curve(ToneCurveInterpolation::Natural)),
                blue: Some(tone_curve(ToneCurveInterpolation::Monotonic)),
            },
        },
    );
    let plan = resolved(&mixed);
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    let natural = "curves=master='0/0 0.5/0.55 1/1':green='0/0 0.5/0.55 1/1':interp=natural";
    let monotonic = "curves=red='0/0 0.5/0.55 1/1':blue='0/0 0.5/0.55 1/1':interp=pchip";
    let natural_at = graph
        .find(natural)
        .unwrap_or_else(|| panic!("missing {natural}: {graph}"));
    let monotonic_at = graph
        .find(monotonic)
        .unwrap_or_else(|| panic!("missing {monotonic}: {graph}"));
    assert!(natural_at < monotonic_at, "master must be applied first");
}

#[test]
fn monotonic_master_precedes_a_natural_channel_curve() {
    let project = graded_project(MaterialKind::Lut3d, LutInterpolation::Tetrahedral);
    let mut plan = resolved(&project);
    let pipeline = plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .color_pipeline
        .as_mut()
        .unwrap();
    let ResolvedColorStage::Curves { curves } = &mut pipeline.stages[2] else {
        unreachable!()
    };
    curves.red = Some(tone_curve(ToneCurveInterpolation::Natural));
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    let master = "curves=master='0/0 0.5/0.55 1/1':interp=pchip";
    let red = "curves=red='0/0 0.5/0.55 1/1':interp=natural";
    let master_at = graph
        .find(master)
        .unwrap_or_else(|| panic!("missing {master}: {graph}"));
    let red_at = graph
        .find(red)
        .unwrap_or_else(|| panic!("missing {red}: {graph}"));
    assert!(master_at < red_at, "master must be applied first");
}

#[test]
fn mismatched_resolved_lut_fails_closed_as_an_invalid_plan() {
    let project = graded_project(MaterialKind::Lut3d, LutInterpolation::Tetrahedral);
    let mut plan = resolved(&project);
    let pipeline = plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .color_pipeline
        .as_mut()
        .unwrap();
    let ResolvedColorStage::Lut { application } = pipeline.stages.last_mut().unwrap() else {
        unreachable!()
    };
    application.interpolation = LutInterpolation::Linear;
    let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();
    assert_eq!(error.diagnostics()[0].kind, CodegenErrorKind::InvalidPlan);
    assert_eq!(error.diagnostics()[0].code, "PLAN_COLOR_PIPELINE_INVALID");
}

#[test]
fn pq_and_hlg_pipeline_conversions_use_zscale_supported_transfers() {
    for (transfer, marker) in [
        (ColorTransfer::Smpte2084, "transfer=smpte2084"),
        (ColorTransfer::AribStdB67, "transfer=arib-std-b67"),
    ] {
        let project = graded_project(MaterialKind::Lut3d, LutInterpolation::Tetrahedral);
        let mut plan = resolved(&project);
        plan.sequences[0].tracks[0].clips[0]
            .visual
            .as_mut()
            .unwrap()
            .color_pipeline
            .as_mut()
            .unwrap()
            .working = ColorSpace {
            primaries: ColorPrimaries::Bt2020,
            transfer,
            matrix: ColorMatrix::Bt2020Ncl,
            range: ColorRange::Limited,
        };
        let graph = emit_video_command(&plan, &bindings(&plan))
            .unwrap()
            .filter_graph
            .unwrap();
        assert!(graph.contains("zscale="), "graph={graph}");
        assert!(graph.contains(marker), "missing {marker}: {graph}");
        assert!(graph.contains("format=yuv444p16le"), "graph={graph}");
        assert!(
            graph.contains("alphaextract,format=gray16le"),
            "graph={graph}"
        );
        assert!(graph.contains("mergeplanes=format="), "graph={graph}");
    }
}
