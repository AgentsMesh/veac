use std::collections::BTreeMap;

use veac_artifact::validate_artifact_json;
use veac_codegen::emitter::MAX_INLINE_FILTER_GRAPH_BYTES;
use veac_ir::{Animatable, Interpolation, TextAnimation, TextGranularity, TextUnitTransform};

use super::support::*;
use super::text_advanced::text_advanced_fixtures::{
    add_font_materials, add_text_scene, advanced_style, key,
};

#[test]
fn animated_text_renders_through_a_large_filter_script() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("text-script.mp4");
    let mut canonical = project(false);
    add_font_materials(&mut canonical, &["filter_script_font"]);
    canonical.project.sequences[0].settings.frame_rate = ratio(30, 1);
    canonical.project.render_configs[0].frame_rate = ratio(30, 1);
    let mut style = advanced_style("filter_script_font", 12.0);
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Grapheme,
        transform: TextUnitTransform::default(),
        reveal: Animatable::Keyframes {
            keyframes: vec![
                key("kf_script_a", 0, 0.0, Interpolation::Hold),
                key("kf_script_b", 600, 1.0, Interpolation::Linear),
            ],
        },
        opacity: Animatable::Keyframes {
            keyframes: vec![
                key("kf_script_opacity_a", 0, 0.0, Interpolation::Linear),
                key("kf_script_opacity_b", 600, 1.0, Interpolation::Linear),
            ],
        },
        stagger: time(60),
        highlight: None,
    });
    add_text_scene(
        &mut canonical,
        text_clip(
            "itm_filter_script",
            "Every word arrives with intent",
            style,
            0,
            5_000,
        ),
        5_000,
    );
    let assets = BTreeMap::from([("med_filter_script_font".to_owned(), font_fixture())]);
    let rendered = render(canonical, &assets, &output);

    let value = serde_json::to_value(&rendered.plan).unwrap();
    validate_artifact_json(&value).unwrap();
    let round_trip: veac_plan::ResolvedRenderPlan =
        serde_json::from_slice(&serde_json::to_vec(&rendered.plan).unwrap()).unwrap();
    assert_eq!(round_trip, rendered.plan);
    assert!(rendered.command.filter_graph.as_ref().unwrap().len() > MAX_INLINE_FILTER_GRAPH_BYTES);
    assert_eq!(video_frame_count(&output), 150);
    assert!(output.metadata().unwrap().len() > 1_000);
}
