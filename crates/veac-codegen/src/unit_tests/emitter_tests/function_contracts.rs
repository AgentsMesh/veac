use veac_codegen::emitter::{BackendPhase, BackendProduct};
use veac_plan::canonical::{BlendMode, TrackMatteMode};

use super::composition_advanced::advanced_plan;
use super::support::{bindings, emit_video_command, visual};

#[test]
fn backend_phase_and_product_labels_are_stable() {
    assert_eq!(
        [
            BackendPhase::Single.as_str(),
            BackendPhase::FirstPass.as_str(),
            BackendPhase::SecondPass.as_str(),
        ],
        ["single", "pass_1", "pass_2"]
    );
    assert_eq!(
        [
            BackendProduct::VideoMaster.as_str(),
            BackendProduct::RenderPassLog.as_str(),
            BackendProduct::ImageSequence.as_str(),
            BackendProduct::CaptionSidecar.as_str(),
            BackendProduct::AudioStem.as_str(),
            BackendProduct::VideoWaveform.as_str(),
            BackendProduct::Vectorscope.as_str(),
            BackendProduct::Histogram.as_str(),
        ],
        [
            "video_master",
            "render_pass_log",
            "image_sequence",
            "caption_sidecar",
            "audio_stem",
            "video_waveform",
            "vectorscope",
            "histogram",
        ]
    );
}

#[test]
fn matte_modes_and_card_shadow_emit_observable_filter_graphs() {
    let alpha = graph(TrackMatteMode::Alpha, false, true);
    for marker in [
        "mattetrimv",
        "mattetargetsplitv",
        "alphaextract,format=gray16le",
        "blend=all_mode=multiply",
        "mattemergev",
        "cardsplit",
        "pad=iw+32:ih+32:16:16:color=black@0",
        "gblur=sigma=8:planes=8",
        "shadowv",
    ] {
        assert!(alpha.contains(marker), "missing {marker}: {alpha}");
    }
    assert!(!alpha.contains("matteinvertv"), "graph={alpha}");

    let inverted_luma = graph(TrackMatteMode::Luma, true, false);
    for marker in [
        "mattevaluev",
        "format=gray16le",
        "matteinvertv",
        "mattemergev",
    ] {
        assert!(
            inverted_luma.contains(marker),
            "missing {marker}: {inverted_luma}"
        );
    }
    assert!(!inverted_luma.contains("shadowv"), "graph={inverted_luma}");
}

#[test]
fn non_normal_blends_preserve_and_combine_base_and_layer_alpha() {
    let mut plan = advanced_plan();
    plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .compositing
        .blend_mode = BlendMode::Multiply;

    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();

    for marker in [
        "format=gbrp16le",
        "blendbasealphav",
        "blendstraightv",
        "blendmergev",
        "premultiply=planes=7",
        "unpremultiply=planes=7",
        "overoutputalphav",
    ] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
}

fn graph(mode: TrackMatteMode, invert: bool, shadow: bool) -> String {
    let mut plan = advanced_plan();
    let target = plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .expect("advanced fixture has target visuals");
    let matte = target
        .track_matte
        .as_mut()
        .expect("advanced fixture has a track matte");
    matte.mode = mode;
    matte.invert = invert;
    target.card = shadow.then(|| visual().card.expect("visual fixture has a card"));
    emit_video_command(&plan, &bindings(&plan))
        .expect("valid matte fixture emits")
        .filter_graph
        .expect("video delivery has a filter graph")
}
