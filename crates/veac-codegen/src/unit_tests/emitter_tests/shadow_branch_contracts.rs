use veac_plan::canonical::{BlendMode, Color};
use veac_plan::{ResolvedApplyItem, ResolvedApplyTarget};

use super::composition_advanced::advanced_plan;
use super::support::{bindings, emit_video_command, visual};

#[test]
fn matte_and_item_apply_process_both_shadow_branches_before_author_blend() {
    let mut plan = advanced_plan();
    let sequence = &mut plan.sequences[0];
    let range = sequence.applies[0].record_range;
    let track_id = sequence.tracks[0].id.clone();
    let item_id = sequence.tracks[0].clips[0].id.clone();
    let target = &mut sequence.tracks[0].clips[0];
    target.effects.clear();
    let target_visual = target.visual.as_mut().unwrap();
    target_visual.card = visual().card;
    target_visual.compositing.blend_mode = BlendMode::Screen;
    let shadow = target_visual
        .card
        .as_mut()
        .unwrap()
        .shadow
        .as_mut()
        .unwrap();
    shadow.color = Color {
        red: 17,
        green: 34,
        blue: 51,
        alpha: 128,
    };
    shadow.opacity = 0.5;
    sequence.applies[0].target = ResolvedApplyTarget::ItemSet {
        items: vec![ResolvedApplyItem {
            track_id,
            item_id,
            active_range: range,
        }],
    };

    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    assert_eq!(
        graph.matches("blend=all_mode=multiply").count(),
        2,
        "track matte must process shadow and foreground: {graph}"
    );
    assert!(
        graph.contains(
            "format=gbrap16le,pad=iw+32:ih+32:16:16:color=black@0,\
             geq=r=4369:g=8738:b=13107:a='alpha(X,Y)*0.250980392157'"
        ),
        "shadow must preserve 16-bit precision and expand RGB channels: {graph}"
    );
    assert_eq!(
        graph.matches("gblur@").count(),
        2,
        "item apply must process shadow and foreground: {graph}"
    );
    assert!(
        graph
            .split(';')
            .any(|node| { node.starts_with("[shadowoffsetv") && node.contains("[sop") }),
        "shadow must enter Normal source-over composition: {graph}"
    );
    assert!(
        graph.contains("blend=all_mode=screen"),
        "foreground must retain its authored blend mode: {graph}"
    );
}
