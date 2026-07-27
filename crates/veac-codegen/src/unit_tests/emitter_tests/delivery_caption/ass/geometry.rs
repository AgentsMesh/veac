use super::super::*;
use super::support::normalize;

#[test]
fn every_ass_alignment_and_placement_anchor_is_emitted() {
    use HorizontalTextAlignment as H;
    use VerticalTextAlignment as V;
    for (horizontal, vertical, tag) in [
        (H::Left, V::Bottom, 1),
        (H::Center, V::Bottom, 2),
        (H::Right, V::Bottom, 3),
        (H::Left, V::Middle, 4),
        (H::Center, V::Middle, 5),
        (H::Right, V::Middle, 6),
        (H::Left, V::Top, 7),
        (H::Center, V::Top, 8),
        (H::Right, V::Top, 9),
    ] {
        let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Ass);
        normalize(&mut plan);
        for clip in captions(&mut plan) {
            let ResolvedClipSource::Caption { content, .. } = &mut clip.source else {
                unreachable!()
            };
            content.style.layout.horizontal_alignment = horizontal;
            content.style.layout.vertical_alignment = vertical;
        }
        assert!(rendered(&plan, &bindings).contains(&format!("\\an{tag}")));
    }

    for anchor in [
        Anchor::TopLeft,
        Anchor::Top,
        Anchor::TopRight,
        Anchor::Left,
        Anchor::Center,
        Anchor::Right,
        Anchor::BottomLeft,
        Anchor::Bottom,
        Anchor::BottomRight,
    ] {
        let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Ass);
        normalize(&mut plan);
        for clip in captions(&mut plan) {
            clip.visual.as_mut().unwrap().placement = Placement::Anchor {
                anchor,
                inset: Vec2 { x: 4.0, y: 6.0 },
            };
        }
        assert!(rendered(&plan, &bindings).contains("\\pos("));
    }
}

#[test]
fn absolute_units_and_style_variants_have_stable_ass_output() {
    for (unit, value) in [
        (LengthUnit::Pixels, 10.0),
        (LengthUnit::Normalized, 0.25),
        (LengthUnit::Percent, 25.0),
    ] {
        let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Ass);
        normalize(&mut plan);
        for clip in captions(&mut plan) {
            clip.visual.as_mut().unwrap().placement = Placement::Absolute {
                position: Point {
                    x: Length { value, unit },
                    y: Length { value, unit },
                },
            };
        }
        assert!(rendered(&plan, &bindings).contains("\\pos("));
    }

    let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Ass);
    normalize(&mut plan);
    let clips = captions(&mut plan);
    let ResolvedClipSource::Caption { content, .. } = &mut clips[0].source else {
        unreachable!()
    };
    content.style.size_pixels = 33.0;
    content.style.outline = None;
    content.style.shadow = None;
    let output = rendered(&plan, &bindings);
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("Style: VEAC"))
            .count(),
        2
    );
    assert!(output.contains("\\shad0"));
}

fn captions(plan: &mut ResolvedRenderPlan) -> &mut [veac_plan::ResolvedClip] {
    &mut plan.sequences[0]
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap()
        .clips
}

fn rendered(plan: &ResolvedRenderPlan, bindings: &ExecutionBindings) -> String {
    content(&emit_all(plan, bindings).unwrap())
}
