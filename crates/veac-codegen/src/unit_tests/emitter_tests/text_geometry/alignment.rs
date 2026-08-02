use super::*;

#[test]
fn horizontal_lines_use_native_ass_anchors() {
    for (alignment, anchor, x) in [
        (HorizontalTextAlignment::Left, 7, 660),
        (HorizontalTextAlignment::Center, 8, 960),
        (HorizontalTextAlignment::Right, 9, 1_260),
    ] {
        let mut plan = resolved(&text_fixture(false));
        let style = text_content(&mut plan).styled_mut().unwrap();
        style.layout.box_width_pixels = Some(600.0);
        style.layout.box_height_pixels = Some(200.0);
        style.layout.overflow = TextOverflow::Visible;
        style.layout.horizontal_alignment = alignment;
        let graph = emit_video_command(&plan, &bindings(&plan))
            .unwrap()
            .filter_graph
            .unwrap();
        let ass = ass_script(&graph);
        let fills = events(&ass, 2);
        let shadows = events(&ass, 1);
        assert_eq!((fills.len(), shadows.len()), (2, 2));
        for fill in fills {
            assert!(
                fill.contains(&format!("\\an{anchor}\\q2\\pos({x},")),
                "{fill}"
            );
        }
        for shadow in shadows {
            assert!(
                shadow.contains(&format!("\\an{anchor}\\q2\\pos({},", x + 2)),
                "{shadow}"
            );
        }
        for background in events(&ass, 0) {
            assert!(background.contains("\\an7\\pos(0,0)\\p1"), "{background}");
        }
    }
}

fn events(ass: &str, layer: u8) -> Vec<&str> {
    let prefix = format!("Dialogue: {layer},");
    ass.lines()
        .filter(|line| line.starts_with(&prefix))
        .collect()
}
