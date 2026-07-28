use super::support::*;
use super::text_advanced::text_advanced_fixtures::{
    add_font_materials, add_text_scene, advanced_style,
};
use tempfile::tempdir;

const DELIVERY_WIDTH: u32 = 192;
const DELIVERY_HEIGHT: u32 = 108;

#[test]
fn blurred_shadow_keeps_a_bright_fill_after_delivery_scale() {
    let font_id = "font-shadow";
    let font = font_fixture();
    let mut style = advanced_style(font_id, 112.0);
    style.outline = None;
    style.shadow = Some(Shadow {
        color: color(255, 0, 0),
        opacity: 1.0,
        offset: Vec2 { x: 0.0, y: 24.0 },
        blur_pixels: 18.0,
    });
    let mut canonical = project(false);
    canonical.project.render_configs[0].width = DELIVERY_WIDTH;
    canonical.project.render_configs[0].height = DELIVERY_HEIGHT;
    canonical.project.sequences[0].settings.width = 768;
    canonical.project.sequences[0].settings.height = 432;
    add_text_scene(
        &mut canonical,
        text_clip("itm_title", "VEAC", style, 0, 1_000),
        1_000,
    );
    add_font_materials(&mut canonical, &[font_id]);
    let materials = BTreeMap::from([(format!("med_{font_id}"), font)]);
    let temp = tempdir().expect("shadow pixel tempdir");
    let output = temp.path().join("shadow-pixels.mp4");

    render(canonical, &materials, &output);
    let frame = rgb_frame_sized(&output, 0.5, DELIVERY_WIDTH, DELIVERY_HEIGHT);
    let mut bright_count = 0usize;
    let mut brightest_floor = 0u8;
    let mut bright_y = 0usize;
    for (index, pixel) in frame.chunks_exact(3).enumerate() {
        let [red, green, blue] = [pixel[0], pixel[1], pixel[2]];
        brightest_floor = brightest_floor.max(red.min(green).min(blue));
        if red >= 210 && green >= 210 && blue >= 210 {
            bright_count += 1;
            let y = index / DELIVERY_WIDTH as usize;
            bright_y += y;
        }
    }

    let bright_centroid = bright_y as f64 / bright_count.max(1) as f64;
    let mut red_count = 0usize;
    let mut red_y = 0usize;
    for (index, pixel) in frame.chunks_exact(3).enumerate() {
        let y = index / DELIVERY_WIDTH as usize;
        let dominance = pixel[0].saturating_sub(pixel[1].max(pixel[2]));
        if pixel[0] >= 16 && dominance >= 10 {
            red_count += 1;
            red_y += y;
        }
    }
    let red_centroid = red_y as f64 / red_count.max(1) as f64;
    assert!(
        brightest_floor >= 220
            && bright_count >= 8
            && red_count >= 32
            && red_centroid > bright_centroid + 5.0,
        "floor={brightest_floor} bright={bright_count}@{bright_centroid} \
         red shadow={red_count}@{red_centroid}"
    );
}
