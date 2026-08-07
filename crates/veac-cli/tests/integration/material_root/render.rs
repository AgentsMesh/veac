use super::super::support::video_dimensions;
use super::support::DetachedFixture;

#[test]
fn detached_render_keeps_outputs_and_artifacts_beside_ir() {
    let fixture = DetachedFixture::new();
    fixture.build_success();
    let project_before = std::fs::read(&fixture.project).unwrap();
    let media_before = std::fs::read(&fixture.media).unwrap();
    fixture
        .command("render")
        .args(["--proxy-policy", "original"])
        .args(["--render-segment-policy", "original"])
        .assert()
        .success();

    let rendered = fixture.build_root.join("media-render.mp4");
    assert_eq!(video_dimensions(&rendered), "32x24");
    assert!(fixture.build_root.join(".veac-artifacts").is_dir());
    assert!(!fixture.source_root.join("media-render.mp4").exists());
    assert!(!fixture.source_root.join(".veac-artifacts").exists());
    assert_eq!(std::fs::read(&fixture.project).unwrap(), project_before);
    assert_eq!(std::fs::read(&fixture.media).unwrap(), media_before);
}
