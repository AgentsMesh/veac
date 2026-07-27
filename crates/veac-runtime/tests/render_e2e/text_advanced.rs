use tempfile::tempdir;

use super::support::*;
use text_advanced_fixtures::*;

pub(crate) mod text_advanced_fixtures;

#[test]
fn unicode_fallback_rtl_emoji_and_rich_span_render_real_glyphs() {
    let fonts = unicode_fonts();
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    add_font_materials(&mut canonical, &["latin", "emoji", "cjk", "rtl", "alt"]);
    let mut style = advanced_style("latin", 19.0);
    style.fallback_fonts = ["emoji", "cjk", "rtl"].map(font_ref).into_iter().collect();
    style.spans.push(TextSpan {
        start: 1,
        end: 2,
        font: Some(font_ref("alt")),
        font_weight: Some(FontWeight::Bold),
        font_style: None,
        size_pixels: Some(22.0),
        color: Some(color(255, 20, 20)),
    });
    let text = text_clip("itm_unicode", "A中ש☺", style, 0, 1_000);
    add_text_scene(&mut canonical, text, 1_000);
    let output = temp.path().join("unicode.mp4");
    let rendered = render(canonical, &fonts, &output);
    assert!(rendered
        .command
        .filter_graph
        .as_deref()
        .unwrap()
        .contains("subtitles=filename="));
    let frame = rgb_frame(&output, 0.5);
    let stats = frame_stats(&frame);
    assert!(stats.ratio > 0.008, "stats={stats:?}");
    let red = frame
        .chunks_exact(3)
        .filter(|pixel| pixel[0] > 130 && u16::from(pixel[0]) > u16::from(pixel[1]) * 2)
        .count();
    assert!(red > 3, "rich red pixels={red}");
}

#[test]
fn cross_directory_custom_font_renders_from_inline_ass_attachment() {
    let temp = tempdir().unwrap();
    let custom = temp.path().join("custom.ttf");
    std::fs::copy(small_script_font(), &custom).unwrap();
    let font = font_fixture();
    let mut canonical = project(false);
    add_font_materials(&mut canonical, &["font", "custom"]);
    let mut style = advanced_style("font", 24.0);
    style.fallback_fonts.push(font_ref("custom"));
    let text = text_clip("itm_embedded_font", "\u{108e0}", style, 0, 1_000);
    add_text_scene(&mut canonical, text, 1_000);
    let output = temp.path().join("embedded-font.mp4");
    let assets = BTreeMap::from([
        ("med_font".to_owned(), font),
        ("med_custom".to_owned(), custom),
    ]);
    let rendered = render(canonical, &assets, &output);
    assert!(rendered
        .command
        .filter_graph
        .as_deref()
        .unwrap()
        .contains("base64\\,"));
    let stats = frame_stats(&rgb_frame(&output, 0.5));
    assert!(stats.ratio > 0.002, "embedded glyph missing: {stats:?}");
}

#[test]
fn word_wrap_ellipsis_alignment_tracking_and_line_height_are_visible() {
    let temp = tempdir().unwrap();
    let font = font_fixture();
    let mut canonical = project(false);
    add_font_materials(&mut canonical, &["font"]);
    let mut style = advanced_style("font", 13.0);
    style.tracking_pixels = 2.0;
    style.line_height = 1.4;
    style.layout = TextLayout {
        box_width_pixels: Some(58.0),
        box_height_pixels: Some(28.0),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Ellipsis,
        horizontal_alignment: HorizontalTextAlignment::Right,
        vertical_alignment: VerticalTextAlignment::Bottom,
        ..TextLayout::default()
    };
    let text = text_clip(
        "itm_ellipsis",
        "alpha beta gamma delta epsilon",
        style,
        0,
        1_000,
    );
    add_text_scene(&mut canonical, text, 1_000);
    let output = temp.path().join("ellipsis.mp4");
    let assets = BTreeMap::from([("med_font".to_owned(), font)]);
    render(canonical, &assets, &output);
    let stats = frame_stats(&rgb_frame(&output, 0.5));
    assert!(stats.ratio > 0.005, "stats={stats:?}");
    assert!(stats.centroid_x > 43.0, "right aligned: {stats:?}");
    assert!(stats.centroid_y > 24.0, "bottom aligned: {stats:?}");
}

#[test]
fn weight_style_and_tracking_change_real_glyph_pixels() {
    let temp = tempdir().unwrap();
    let font = font_fixture();
    let assets = BTreeMap::from([("med_font".to_owned(), font)]);
    let mut normal = project(false);
    add_font_materials(&mut normal, &["font"]);
    let text = text_clip("itm_normal", "VEAC", advanced_style("font", 18.0), 0, 1_000);
    add_text_scene(&mut normal, text, 1_000);
    let normal_output = temp.path().join("normal.mp4");
    render(normal, &assets, &normal_output);

    let mut styled = project(false);
    add_font_materials(&mut styled, &["font"]);
    let mut style = advanced_style("font", 18.0);
    style.font_weight = FontWeight::Bold;
    style.font_style = FontStyle::Italic;
    style.tracking_pixels = 3.0;
    let text = text_clip("itm_styled", "VEAC", style, 0, 1_000);
    add_text_scene(&mut styled, text, 1_000);
    let styled_output = temp.path().join("styled.mp4");
    render(styled, &assets, &styled_output);

    let normal = rgb_frame(&normal_output, 0.5);
    let styled = rgb_frame(&styled_output, 0.5);
    let changed = normal
        .iter()
        .zip(&styled)
        .filter(|(left, right)| u8::abs_diff(**left, **right) > 20)
        .count();
    assert!(changed > 100, "changed channels={changed}");
    assert!(lit_width(&styled) > lit_width(&normal) + 5);
}

#[test]
fn character_reveal_and_stagger_hide_fill_outline_and_shadow() {
    let temp = tempdir().unwrap();
    let font = font_fixture();
    let mut canonical = project(false);
    add_font_materials(&mut canonical, &["font"]);
    let mut style = advanced_style("font", 20.0);
    style.outline = Some(TextOutline {
        color: color(255, 255, 255),
        width_pixels: 2.0,
    });
    style.shadow = Some(Shadow {
        blur_pixels: 2.0,
        opacity: 1.0,
        offset: Vec2 { x: 3.0, y: 2.0 },
        color: color(255, 255, 255),
    });
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Grapheme,
        transform: TextUnitTransform::default(),
        reveal: Animatable::Keyframes {
            keyframes: vec![
                key("kf_reveal_a", 0, 0.0, Interpolation::Hold),
                key("kf_reveal_b", 100, 1.0, Interpolation::Linear),
            ],
        },
        opacity: Animatable::Keyframes {
            keyframes: vec![
                key("kf_opacity_a", 0, 0.0, Interpolation::Linear),
                key("kf_opacity_b", 300, 1.0, Interpolation::Linear),
            ],
        },
        stagger: time(100),
        highlight: None,
    });
    let text = text_clip("itm_reveal", "VEAC", style, 0, 1_000);
    add_text_scene(&mut canonical, text, 1_000);
    let output = temp.path().join("reveal.mp4");
    let assets = BTreeMap::from([("med_font".to_owned(), font)]);
    render(canonical, &assets, &output);
    let early = frame_stats(&rgb_frame(&output, 0.02));
    let middle = frame_stats(&rgb_frame(&output, 0.42));
    let late = frame_stats(&rgb_frame(&output, 0.85));
    assert!(early.ratio < 0.001, "outline/shadow leaked: {early:?}");
    assert!(
        middle.energy > early.energy + 1.0,
        "{early:?} -> {middle:?}"
    );
    assert!(late.energy > middle.energy * 1.15, "{middle:?} -> {late:?}");
}
