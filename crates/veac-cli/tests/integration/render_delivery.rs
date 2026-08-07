use super::support::*;

const MULTI_DELIVERY_SOURCE: &str = include_str!("../fixtures/render-delivery.veac");

#[test]
fn real_render_emits_and_resumes_every_authored_deliverable() {
    let temp = tempdir().unwrap();
    let project = compile_ir(&temp, MULTI_DELIVERY_SOURCE);
    let destination = temp.path().join("deliveries");
    std::fs::create_dir(&destination).unwrap();
    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--destination",
            destination.to_str().unwrap(),
        ])
        .assert()
        .success();
    let master = destination.join("authored-master.mp4");
    let first_frame = destination.join("authored-frame-0001.png");
    assert_eq!(video_dimensions(&master), "32x24");
    assert_eq!(video_dimensions(&first_frame), "32x24");
    let frames = std::fs::read_dir(&destination)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("authored-frame-")
        })
        .count();
    assert_eq!(frames, 2);

    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--destination",
            destination.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("reused"));
}
