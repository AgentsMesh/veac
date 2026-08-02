use veac_ir::{Clip, ClipSource, ColorStage, Generator, Gradient, LutInterpolation};

use crate::support::{clips, lower_example};

#[test]
fn basic_color_grade_compares_the_same_source_before_and_after() {
    let envelope = lower_example("color-grade/main.veac");
    let clips: Vec<_> = clips(&envelope).collect();
    assert_eq!(clips.len(), 2);
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
    let main = sequence(&envelope, "seq_main");
    let segments = &main
        .tracks
        .iter()
        .find(|track| track.id.as_str() == "trk_comparisons")
        .expect("advanced-color comparison track")
        .clips;
    assert_eq!(
        segments
            .iter()
            .map(|clip| clip.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "itm_reference",
            "itm_hsl-only",
            "itm_curves-only",
            "itm_wheels-only",
            "itm_full-grade",
            "itm_tone-curve",
        ]
    );
    assert_eq!(
        segments
            .iter()
            .map(|clip| (
                clip.record_range.start.value,
                clip.record_range.duration.value
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, 1_000),
            (1_000, 1_000),
            (2_000, 1_000),
            (3_000, 1_000),
            (4_000, 2_000),
            (6_000, 2_000),
        ]
    );
    assert!(matches!(&segments[0].source,
        ClipSource::Sequence { sequence_id } if sequence_id.as_str() == "seq_color-card"));
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
        matches!(stages(&segments[1]), [ColorStage::Hsl { adjustment }]
        if adjustment.hue_degrees != 0.0
            && adjustment.saturation != 0.0
            && adjustment.lightness != 0.0)
    );
    assert!(
        matches!(stages(&segments[2]), [ColorStage::Curves { curves }]
        if curves.luma.is_some() && curves.red.is_some())
    );
    assert!(
        matches!(stages(&segments[3]), [ColorStage::Wheels { wheels }]
        if wheels.lift.blue != 0.0
            && wheels.gamma.red != 0.0
            && wheels.gain.red != 0.0)
    );

    let full = stages(&segments[4]);
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
        if application.material_id.as_str() == "med_cinematic"
            && application.interpolation == LutInterpolation::Tetrahedral));
    assert!(
        matches!(stages(&segments[5]), [ColorStage::Lut { application }]
        if application.material_id.as_str() == "med_tone-curve"
            && application.interpolation == LutInterpolation::Linear)
    );
}

#[test]
fn advanced_color_places_translucent_input_over_a_contrast_reference() {
    let envelope = lower_example("advanced-color/main.veac");
    let card = sequence(&envelope, "seq_color-card");
    let card_stops = linear_stops(&card.tracks[0].clips[0]);
    assert!(card_stops.iter().all(|stop| stop.color.alpha == 179));

    let main = sequence(&envelope, "seq_main");
    let background = &main
        .tracks
        .iter()
        .find(|track| track.id.as_str() == "trk_background")
        .expect("alpha contrast background")
        .clips;
    assert_eq!(
        background.len(),
        1,
        "alpha proof must use one background source"
    );
    let stops = linear_stops(&background[0]);
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

fn sequence<'a>(envelope: &'a veac_ir::ProjectEnvelope, id: &str) -> &'a veac_ir::Sequence {
    envelope
        .project
        .sequences
        .iter()
        .find(|sequence| sequence.id.as_str() == id)
        .unwrap_or_else(|| panic!("missing sequence {id}"))
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
