use super::support::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn fully_off_canvas_layer_casts_shadow_back_onto_canvas() {
    let temp = tempdir().unwrap();
    let project = shadow_project(point(-24.0, 20.0), Vec2 { x: 30.0, y: 0.0 }, 2.0);
    let output = temp.path().join("shadow-fully-off-canvas.mp4");
    let rendered = render(project, &BTreeMap::new(), &output);
    let frame = rgba_frame(&rendered, 96, 54);
    let shadow = rgba_pixel(&frame, 96, 16, 26);
    let untouched = rgba_pixel(&frame, 96, 48, 27);

    assert!(
        shadow[0] > 180 && shadow[1] < 80 && shadow[2] < 80,
        "off-canvas source shadow must remain visible, got {shadow:?}"
    );
    assert!(
        untouched[..3].iter().all(|channel| *channel < 20),
        "background reference unexpectedly changed: {untouched:?}"
    );
}

#[test]
fn partially_clipped_layer_keeps_blur_continuous_inside_canvas() {
    let temp = tempdir().unwrap();
    let project = shadow_project(point(-6.0, 9.0), Vec2 { x: 0.0, y: 18.0 }, 3.0);
    let output = temp.path().join("shadow-partially-clipped.mp4");
    let rendered = render(project, &BTreeMap::new(), &output);
    let frame = rgba_frame(&rendered, 96, 54);
    let near_clipped_edge = rgba_pixel(&frame, 96, 1, 33);
    let interior = rgba_pixel(&frame, 96, 8, 33);

    assert!(
        near_clipped_edge[0] > 180 && near_clipped_edge[1] < 80 && near_clipped_edge[2] < 80,
        "off-canvas alpha must contribute to the visible blur: {near_clipped_edge:?}"
    );
    assert!(
        near_clipped_edge[1].abs_diff(interior[1]) < 24
            && near_clipped_edge[2].abs_diff(interior[2]) < 24,
        "blur discontinuity: edge={near_clipped_edge:?}, interior={interior:?}"
    );
}

#[test]
fn shadow_color_alpha_multiplies_shadow_opacity_in_real_pixels() {
    let temp = tempdir().unwrap();
    let project =
        shadow_project_with_alpha(point(8.0, 20.0), Vec2 { x: 40.0, y: 0.0 }, 0.0, 128, 0.5);
    let output = temp.path().join("shadow-alpha-opacity.mp4");
    let rendered = render(project, &BTreeMap::new(), &output);
    let frame = rgba_frame(&rendered, 96, 54);
    let shadow = rgba_pixel(&frame, 96, 58, 26);
    let foreground = rgba_pixel(&frame, 96, 18, 26);

    assert!(
        (55..=72).contains(&shadow[0]) && shadow[1] <= 4 && shadow[2] <= 4,
        "128/255 color alpha times 0.5 opacity should produce quarter-red, got {shadow:?}"
    );
    assert!(
        foreground[..3].iter().all(|channel| *channel >= 250),
        "shadow alpha must not change the foreground: {foreground:?}"
    );
}

fn shadow_project(center: Point, offset: Vec2, blur_pixels: f64) -> ProjectEnvelope {
    shadow_project_with_alpha(center, offset, blur_pixels, 255, 1.0)
}

fn shadow_project_with_alpha(
    center: Point,
    offset: Vec2,
    blur_pixels: f64,
    color_alpha: u8,
    opacity: f64,
) -> ProjectEnvelope {
    let mut project = project(false);
    let mut caster = solid_clip("itm_shadow_caster", color(255, 255, 255), 0, 1_000);
    caster.visual = Some(shadow_visual(
        center,
        offset,
        blur_pixels,
        color_alpha,
        opacity,
    ));
    project.project.sequences[0].tracks = vec![track(
        "trk_shadow_caster",
        TrackKind::Video,
        0,
        vec![caster],
    )];
    project
}

fn shadow_visual(
    center: Point,
    offset: Vec2,
    blur_pixels: f64,
    color_alpha: u8,
    opacity: f64,
) -> VisualProperties {
    let mut visual = framed_visual(Anchor::TopLeft, None);
    visual.transform.anchor = Vec2 { x: 0.0, y: 0.0 };
    visual.transform.scale = Animatable::constant(Vec2 {
        x: 20.0 / f64::from(WIDTH),
        y: 12.0 / f64::from(HEIGHT),
    });
    visual.placement = Placement::Absolute { position: center };
    visual.card = Some(CardStyle {
        corner_radius_pixels: 0.0,
        shadow: Some(Shadow {
            color: Color {
                alpha: color_alpha,
                ..color(255, 0, 0)
            },
            offset,
            blur_pixels,
            opacity,
        }),
    });
    visual
}

fn point(x: f64, y: f64) -> Point {
    Point {
        x: pixels(x),
        y: pixels(y),
    }
}

fn rgba_frame(rendered: &Rendered, width: usize, height: usize) -> Vec<u8> {
    assert!(rendered.command.inputs.is_empty());
    let graph = rendered.command.filter_graph.as_deref().unwrap();
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-filter_complex",
            graph,
        ])
        .arg("-map")
        .arg(&rendered.command.maps[0])
        .args([
            "-frames:v",
            "1",
            "-pix_fmt",
            "rgba",
            "-f",
            "rawvideo",
            "pipe:1",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.len(), width * height * 4);
    output.stdout
}

fn rgba_pixel(frame: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * width + x) * 4;
    frame[offset..offset + 4].try_into().unwrap()
}
