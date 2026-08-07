use veac_ir::{
    Clip, ClipSource, ColorStage, Generator, Gradient, LutInterpolation, MaterialId, MaterialKind,
    ProjectEnvelope,
};

use crate::support::{clip_by_key, lower_example};

#[test]
fn basic_color_grade_compares_the_same_source_before_and_after() {
    let envelope = lower_example("color-grade/main.veac");
    let clips = [
        clip_by_key(&envelope, "reference"),
        clip_by_key(&envelope, "balanced"),
    ];
    assert_eq!(clips[0].source, clips[1].source);
    assert!(clips[0]
        .visual
        .as_ref()
        .expect("reference visual properties")
        .color_pipeline
        .is_none());
    assert!(
        matches!(stages(clips[1]), [ColorStage::Basic { adjustment }]
        if adjustment.temperature_kelvin < 6_500.0
            && adjustment.exposure_stops != 0.0
            && adjustment.tint != 0.0
            && adjustment.highlights != 0.0
            && adjustment.shadows != 0.0
            && adjustment.fade != 0.0)
    );
}

#[test]
fn advanced_color_segments_are_independently_attributable() {
    let envelope = lower_example("advanced-color/main.veac");
    let segments = ["reference", "hsl", "curves", "wheels", "full", "tone"]
        .map(|key| clip_by_key(&envelope, key));
    assert_eq!(
        segments
            .iter()
            .map(|clip| (
                clip.record_range.start.value,
                clip.record_range.duration.value
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, 600),
            (600, 600),
            (1_200, 600),
            (1_800, 600),
            (2_400, 1_200),
            (3_600, 1_200),
        ]
    );
    assert!(matches!(&segments[0].source, ClipSource::Generated { .. }));
    assert!(segments
        .iter()
        .all(|clip| clip.source == segments[0].source));
    assert!(segments[0]
        .visual
        .as_ref()
        .expect("reference visual properties")
        .color_pipeline
        .is_none());

    assert!(
        matches!(stages(segments[1]), [ColorStage::Hsl { adjustment }]
        if adjustment.hue_degrees != 0.0
            && adjustment.saturation != 0.0
            && adjustment.lightness != 0.0)
    );
    assert!(
        matches!(stages(segments[2]), [ColorStage::Curves { curves }]
        if curves.luma.is_some() && curves.red.is_some())
    );
    assert!(
        matches!(stages(segments[3]), [ColorStage::Wheels { wheels }]
        if wheels.lift.blue != 0.0
            && wheels.gamma.red != 0.0
            && wheels.gain.red != 0.0)
    );

    let full = stages(segments[4]);
    assert_eq!(full.len(), 6);
    assert!(matches!(&full[0], ColorStage::Basic { adjustment }
        if adjustment.exposure_stops != 0.0
            && adjustment.temperature_kelvin != 6_500.0
            && adjustment.tint != 0.0
            && adjustment.highlights != 0.0
            && adjustment.shadows != 0.0
            && adjustment.fade != 0.0));
    assert!(matches!(&full[1], ColorStage::Matrix { adjustment }
        if adjustment.matrix != [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
            && adjustment.offset != [0.0, 0.0, 0.0]));
    assert!(matches!(&full[2], ColorStage::Hsl { .. }));
    assert!(matches!(&full[3], ColorStage::Curves { .. }));
    assert!(matches!(&full[4], ColorStage::Wheels { .. }));
    assert!(matches!(&full[5], ColorStage::Lut { application }
        if material_kind(&envelope, &application.material_id) == MaterialKind::Lut3d
            && application.interpolation == LutInterpolation::Tetrahedral));
    assert!(
        matches!(stages(segments[5]), [ColorStage::Lut { application }]
        if material_kind(&envelope, &application.material_id) == MaterialKind::Lut1d
            && application.interpolation == LutInterpolation::Linear)
    );
}

#[test]
fn advanced_color_places_translucent_input_over_a_contrast_reference() {
    let envelope = lower_example("advanced-color/main.veac");
    let card_stops = linear_stops(clip_by_key(&envelope, "reference"));
    assert!(card_stops.iter().all(|stop| stop.color.alpha == 179));

    let stops = linear_stops(clip_by_key(&envelope, "alpha-check"));
    assert!(stops.iter().all(|stop| stop.color.alpha == 255));
    assert!(stops.iter().map(|stop| stop.color.red).min().unwrap() <= 5);
    assert!(stops.iter().map(|stop| stop.color.red).max().unwrap() >= 250);
}

fn stages(clip: &Clip) -> &[ColorStage] {
    &clip
        .visual
        .as_ref()
        .expect("color segment visual properties")
        .color_pipeline
        .as_ref()
        .expect("color segment pipeline")
        .stages
}

fn material_kind(envelope: &ProjectEnvelope, id: &MaterialId) -> MaterialKind {
    envelope
        .project
        .materials
        .iter()
        .find(|material| material.id == *id)
        .unwrap_or_else(|| panic!("missing color material {id}"))
        .kind
}

fn linear_stops(clip: &Clip) -> &[veac_ir::GradientStop] {
    let ClipSource::Generated {
        generator:
            Generator::Gradient {
                gradient: Gradient::Linear { stops, .. },
            },
    } = &clip.source
    else {
        panic!("expected generated linear gradient");
    };
    stops
}
