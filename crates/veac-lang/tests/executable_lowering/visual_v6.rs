use veac_ir::{
    Anchor, Animatable, BlendMode, ColorStage, FitMode, Interpolation, MaskShape, Placement,
};

use super::support;

const SOURCE: &str = r#"
fn main(context: Context) -> Project {
    let clip = item(
        identifier("card"), item_enabled(), during(0s, 3s),
        source_generated(generator_solid(#234567ff)), source_timing_native()
    ).with_visual(visual_style(
        visual_layout(
            placement_anchor(anchor_bottom_right(), vector(0.1, 0.2)),
            frame_sized(320px, 180px, fit_cover()),
            transform_2d(
                transform_motion(
                    point_keyframes([
                        point_keyframe(identifier("start"), 0s, point(10px, 20px), interpolation_linear()),
                        point_keyframe(identifier("end"), 2s, point(30px, 40px), interpolation_ease_out())
                    ]),
                    vector_constant(vector(1.2, 0.8)),
                    angle_constant(15deg)
                ),
                transform_geometry(
                    vector(0.1, -0.1), flip_horizontal(), vector(0.25, 0.75),
                    crop_animated(rect_constant(rect(0.1, 0.2, 0.7, 0.6)))
                )
            )
        ),
        visual_surface(
            percent_constant(80%), compositing(3, blend_screen()),
            card_present(12px, shadow_present(4px, 50%, vector(3.0, 4.0), #102030ff))
        ),
        [mask(
            mask_rounded_rectangle(0.2),
            mask_motion(
                vector_constant(vector(0.5, 0.5)),
                vector_constant(vector(1.0, 1.0)), angle_constant(0deg)
            ),
            mask_edge(length_constant(5px), length_constant(-2px)), false
        )],
        color_pipeline_present(color_pipeline(
            color_space(primaries_bt709(), transfer_srgb(), matrix_rgb(), range_full()),
            color_space(primaries_bt709(), transfer_linear(), matrix_rgb(), range_full()),
            color_space(primaries_bt709(), transfer_srgb(), matrix_rgb(), range_full()),
            [color_stage_basic(basic_color(0.5, 6500.0, 0.1, 0.2, -0.2, 0.1))]
        ))
    ));
    let track = visual_layer(
        identifier("visual"), 0, placement_free(),
        track_state(track_playback_enabled(), track_audio_audible(),
            track_isolation_normal(), track_editing_unlocked()),
        track_routing_default()
    ).with_item(clip);
    let sequence = sequence(
        identifier("main"), "主时间线",
        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
    ).with_layer(track);
    project(identifier("demo"), project_settings(600))
        .with_sequence(sequence).entry(sequence)
}
"#;

#[test]
fn complete_v6_visual_algebra_lowers_without_property_projection() {
    let envelope = support::envelope(SOURCE);
    let visual = support::clip(&envelope, 0).visual.as_ref().unwrap();
    assert!(matches!(
        visual.placement,
        Placement::Anchor {
            anchor: Anchor::BottomRight,
            ..
        }
    ));
    assert_eq!(visual.frame.unwrap().fit, FitMode::Cover);
    assert_eq!(visual.compositing.blend_mode, BlendMode::Screen);
    assert_eq!(visual.opacity, Animatable::constant(0.8));
    assert_eq!(
        visual.masks[0].shape,
        MaskShape::RoundedRectangle { radius: 0.2 }
    );
    assert_eq!(visual.masks[0].feather_pixels, Animatable::constant(5.0));
    assert_eq!(visual.masks[0].expansion_pixels, Animatable::constant(-2.0));
    let keys = visual.transform.position.keyframes().unwrap();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0].interpolation, Interpolation::Linear);
    assert_ne!(keys[0].id, keys[1].id);
    let pipeline = visual.color_pipeline.as_ref().unwrap();
    assert!(matches!(pipeline.stages[0], ColorStage::Basic { .. }));
    assert!(veac_ir::validate(&envelope).is_ok());
    let json = veac_ir::canonical_json(&envelope).unwrap();
    assert_eq!(veac_ir::decode_canonical_json(&json).unwrap(), envelope);
}

#[test]
fn visual_keyframe_ids_are_stable_across_repeated_execution() {
    let first = support::envelope(SOURCE);
    let second = support::envelope(SOURCE);
    let keys = |value: &veac_ir::ProjectEnvelope| {
        support::clip(value, 0)
            .visual
            .as_ref()
            .unwrap()
            .transform
            .position
            .keyframes()
            .unwrap()
            .iter()
            .map(|value| value.id.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(keys(&first), keys(&second));
}

#[test]
fn every_closed_color_space_value_lowers_through_a_visual_pipeline() {
    for value in [
        "primaries_bt470m()",
        "primaries_bt470bg()",
        "primaries_smpte170m()",
        "primaries_smpte240m()",
        "primaries_film()",
        "primaries_bt2020()",
        "primaries_smpte428()",
        "primaries_smpte431()",
        "primaries_smpte432()",
    ] {
        assert_color_space(&SOURCE.replace("primaries_bt709()", value));
    }
    for value in [
        "transfer_gamma22()",
        "transfer_gamma28()",
        "transfer_smpte170m()",
        "transfer_smpte240m()",
        "transfer_bt2020_10()",
        "transfer_bt2020_12()",
        "transfer_smpte2084()",
        "transfer_arib_std_b67()",
    ] {
        assert_color_space(&SOURCE.replace("transfer_srgb()", value));
    }
    for value in [
        "matrix_fcc()",
        "matrix_bt470bg()",
        "matrix_smpte170m()",
        "matrix_smpte240m()",
        "matrix_ycgco()",
        "matrix_bt2020_ncl()",
    ] {
        assert_color_space(&SOURCE.replace("matrix_rgb()", value));
    }
}

fn assert_color_space(source: &str) {
    let envelope = support::envelope(source);
    let pipeline = support::clip(&envelope, 0)
        .visual
        .as_ref()
        .unwrap()
        .color_pipeline
        .as_ref()
        .unwrap();
    assert_eq!(pipeline.stages.len(), 1);
    veac_ir::validate(&envelope).unwrap();
}
