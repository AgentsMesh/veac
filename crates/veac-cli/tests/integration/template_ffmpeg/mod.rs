mod binding;
mod fixture;
mod project;

use super::support::*;
use fixture::{assert_pixel, Color, FixtureMedia};

#[test]
fn template_fill_modes_and_aspect_crop_render_observable_pixels() {
    if !fixture::tools_available() {
        eprintln!("skipping template FFmpeg E2E: ffmpeg or ffprobe is unavailable");
        return;
    }
    let temp = tempdir().unwrap();
    let media = FixtureMedia::create(temp.path());
    let template = compile_ir(&temp, project::SOURCE);
    let request = project::write_request(&temp, &template, &media);

    let proposed = veac()
        .args([
            "template",
            "propose",
            template.to_str().unwrap(),
            request.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        proposed.status.success(),
        "template proposal failed: {}",
        String::from_utf8_lossy(&proposed.stderr)
    );
    let batch = veac_ir::decode_edit_batch_json(
        std::str::from_utf8(&proposed.stdout).expect("proposal UTF-8"),
    )
    .expect("proposal must contain only a canonical EditBatch");
    assert!(batch.atomic);
    assert_eq!(batch.operations.len(), 20);

    let batch_file = temp.path().join("template-edit.json");
    std::fs::write(&batch_file, &proposed.stdout).unwrap();
    let filled = temp.path().join("filled-project.json");
    veac()
        .args([
            "edit",
            template.to_str().unwrap(),
            batch_file.to_str().unwrap(),
            "-o",
            filled.to_str().unwrap(),
        ])
        .assert()
        .success();
    project::assert_template_state_cleared(&filled);
    veac()
        .args(["check-ir", filled.to_str().unwrap()])
        .assert()
        .success();

    let plan = veac()
        .args(["plan", filled.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        plan.status.success(),
        "planning failed: {}",
        String::from_utf8_lossy(&plan.stderr)
    );
    serde_json::from_slice::<serde_json::Value>(&plan.stdout).expect("canonical render plan");

    let deliveries = temp.path().join("deliveries");
    std::fs::create_dir(&deliveries).unwrap();
    veac()
        .args([
            "render",
            filled.to_str().unwrap(),
            "--destination",
            deliveries.to_str().unwrap(),
        ])
        .assert()
        .success();
    let output = deliveries.join("template-render.mp4");
    assert_eq!(video_dimensions(&output), "64x36");
    assert!((fixture::duration(&output) - 5.0).abs() < 0.12);

    assert_pixel(&output, 0.15, 32, 18, Color::Red);
    assert_pixel(&output, 0.50, 32, 18, Color::Green);
    assert_pixel(&output, 0.85, 32, 18, Color::Blue);
    assert_pixel(&output, 1.20, 32, 18, Color::Red);
    assert_pixel(&output, 1.80, 32, 18, Color::Red);
    assert_pixel(&output, 2.20, 32, 18, Color::Green);
    assert_pixel(&output, 2.80, 32, 18, Color::Green);
    assert_pixel(&output, 3.50, 2, 18, Color::Red);
    assert_pixel(&output, 3.50, 32, 18, Color::Green);
    assert_pixel(&output, 3.50, 60, 18, Color::Blue);
    assert_pixel(&output, 4.50, 32, 18, Color::Red);
}
