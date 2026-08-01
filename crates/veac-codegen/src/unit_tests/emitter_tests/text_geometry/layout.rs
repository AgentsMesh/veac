use super::*;

#[test]
fn vertical_columns_keep_logical_order_and_explicit_orientation() {
    let right_to_left = vertical_ass(TextWritingMode::VerticalRl);
    let left_to_right = vertical_ass(TextWritingMode::VerticalLr);
    assert!(line(&right_to_left, "中").contains("\\frz0"));
    assert!(line(&right_to_left, "A").contains("\\frz90"));
    assert!(x(line(&right_to_left, "中")) > x(line(&right_to_left, "文")));
    assert!(x(line(&left_to_right, "中")) < x(line(&left_to_right, "文")));
    assert_eq!(right_to_left.matches("Dialogue: 1").count(), 4);
}

#[test]
fn text_path_and_per_grapheme_transform_emit_position_rotation_and_scale() {
    let mut plan = resolved(&text_fixture(false));
    let content = text_content(&mut plan);
    content.text = "AB".to_owned();
    content.styled_mut().unwrap().background = None;
    content.styled_mut().unwrap().path = Some(TextPath {
        points: vec![point(100.0, 200.0), point(400.0, 200.0)],
        start_offset: pixels(250.0),
        reverse: true,
        alignment: TextPathAlignment::Center,
    });
    content.styled_mut().unwrap().animation = Some(TextAnimation {
        granularity: TextGranularity::Grapheme,
        transform: TextUnitTransform {
            position_offset: Animatable::constant(point(5.0, 7.0)),
            scale: Animatable::constant(Vec2 { x: 1.2, y: 0.8 }),
            rotation_degrees: Animatable::constant(15.0),
        },
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::constant(1.0),
        stagger: RationalTime::zero(600).unwrap(),
    });
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    let ass = ass_script(&graph);
    assert_eq!(ass.matches("Dialogue: 1").count(), 2);
    for marker in ["\\pos(", "\\frz195", "\\fscx120", "\\fscy80"] {
        assert!(ass.contains(marker), "missing {marker}: {ass}");
    }
}

#[test]
fn placed_text_draws_background_before_glyphs_and_skips_invisible_backgrounds() {
    let mut plan = resolved(&text_fixture(false));
    let content = text_content(&mut plan);
    content.text = "AB".to_owned();
    content.styled_mut().unwrap().path = Some(TextPath {
        points: vec![point(100.0, 200.0), point(500.0, 200.0)],
        start_offset: pixels(0.0),
        reverse: false,
        alignment: TextPathAlignment::Start,
    });
    let ass = ass_script(
        emit_video_command(&plan, &bindings(&plan))
            .unwrap()
            .filter_graph
            .as_deref()
            .unwrap(),
    );
    assert_eq!(ass.matches("Dialogue: 0").count(), 2);
    assert_eq!(ass.matches("Dialogue: 1").count(), 2);
    assert!(ass.contains("\\p1\\bord0\\shad0"), "{ass}");

    text_content(&mut plan).styled_mut().unwrap().animation = Some(TextAnimation {
        granularity: TextGranularity::Grapheme,
        transform: TextUnitTransform {
            position_offset: Animatable::constant(point(0.0, 0.0)),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            rotation_degrees: Animatable::constant(0.0),
        },
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::constant(0.0),
        stagger: RationalTime::zero(600).unwrap(),
    });
    let ass = ass_script(
        emit_video_command(&plan, &bindings(&plan))
            .unwrap()
            .filter_graph
            .as_deref()
            .unwrap(),
    );
    assert_eq!(ass.matches("Dialogue: 0").count(), 0);
}

#[test]
fn text_path_point_zero_length_and_bounds_fail_closed() {
    let short = path_plan(vec![point(0.0, 0.0)]);
    assert_code(&short, "TEXT_PATH_POINT_LIMIT");

    let zero = path_plan(vec![
        point(0.0, 0.0),
        Point {
            x: Length {
                value: 0.0,
                unit: LengthUnit::Percent,
            },
            y: pixels(0.0),
        },
    ]);
    assert_code(&zero, "TEXT_PATH_ZERO_LENGTH");

    let bounds = path_plan(vec![point(0.0, 0.0), point(1.0, 0.0)]);
    assert_code(&bounds, "TEXT_PATH_BOUNDS");

    let budget = path_plan((0..257).map(|index| point(index as f64, 0.0)).collect());
    assert_code(&budget, "TEXT_PATH_POINT_LIMIT");
}

#[test]
fn malformed_geometry_plans_fail_at_each_backend_budget() {
    let mut multiline = path_plan(vec![point(0.0, 0.0), point(1_000.0, 0.0)]);
    text_content(&mut multiline).text = "A\nB".to_owned();
    assert_code(&multiline, "TEXT_PATH_MULTILINE");

    let mut path_vertical = path_plan(vec![point(0.0, 0.0), point(1_000.0, 0.0)]);
    text_content(&mut path_vertical)
        .styled_mut()
        .unwrap()
        .layout
        .writing_mode = TextWritingMode::VerticalRl;
    assert_code(&path_vertical, "TEXT_PATH_WRITING_MODE");

    let mut invalid_vertical = resolved(&text_fixture(false));
    let style = &mut text_content(&mut invalid_vertical).styled_mut().unwrap();
    style.layout.writing_mode = TextWritingMode::VerticalLr;
    style.layout.wrap = TextWrap::Word;
    assert_code(&invalid_vertical, "TEXT_VERTICAL_LAYOUT_INVALID");

    let mut over_budget = resolved(&text_fixture(false));
    let content = text_content(&mut over_budget);
    content.text = "A".repeat(2_049);
    content.styled_mut().unwrap().background = None;
    content.styled_mut().unwrap().layout.writing_mode = TextWritingMode::VerticalRl;
    assert_code(&over_budget, "TEXT_LAYOUT_UNIT_LIMIT");
}

fn vertical_ass(mode: TextWritingMode) -> String {
    let mut plan = resolved(&text_fixture(false));
    let font = text_content(&mut plan)
        .styled_mut()
        .unwrap()
        .font
        .input_id
        .clone();
    let content = text_content(&mut plan);
    content.text = "中A\n文B".to_owned();
    content.styled_mut().unwrap().background = None;
    content.styled_mut().unwrap().layout.writing_mode = mode;
    content.styled_mut().unwrap().layout.orientation = TextOrientation::Mixed;
    let path = cjk_font_path();
    set_input_identity(&mut plan, &font, &path);
    let mut local = bindings(&plan);
    let input = plan.inputs.iter().find(|input| input.id == font).unwrap();
    local.bind_original(input, path).unwrap();
    let graph = emit_video_command(&plan, &local)
        .unwrap()
        .filter_graph
        .unwrap();
    ass_script(&graph)
}

fn path_plan(points: Vec<Point>) -> veac_plan::ResolvedRenderPlan {
    let mut plan = resolved(&text_fixture(false));
    let content = text_content(&mut plan);
    content.text = "AB".to_owned();
    content.styled_mut().unwrap().background = None;
    content.styled_mut().unwrap().path = Some(TextPath {
        points,
        start_offset: pixels(0.0),
        reverse: false,
        alignment: TextPathAlignment::Start,
    });
    plan
}

fn assert_code(plan: &veac_plan::ResolvedRenderPlan, expected: &str) {
    let error = emit_video_command(plan, &bindings(plan)).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, expected);
}

fn line<'a>(ass: &'a str, text: &str) -> &'a str {
    ass.lines()
        .find(|line| line.ends_with(text))
        .unwrap_or_else(|| panic!("missing {text}: {ass}"))
}

fn x(line: &str) -> f64 {
    let (_, value) = line.split_once("\\pos(").unwrap();
    value.split_once(',').unwrap().0.parse().unwrap()
}
