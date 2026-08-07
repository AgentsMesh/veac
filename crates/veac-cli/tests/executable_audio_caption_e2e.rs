#[path = "executable_audio_caption_e2e/fixture.rs"]
mod fixture;
#[path = "executable_audio_caption_e2e/media.rs"]
mod media;
#[path = "executable_audio_caption_e2e/model.rs"]
mod model;

use tempfile::tempdir;

#[test]
#[ignore = "isolated FFmpeg guard; run make e2e-executable-audio-caption"]
fn executable_audio_caption_preserves_identity_and_renders_timed_evidence() {
    let temp = tempdir().unwrap();
    let paths = fixture::prepare(temp.path());

    let envelope = fixture::build(&paths);
    model::assert_canonical(&envelope);
    let canonical_before_plan = std::fs::read(&paths.project).unwrap();

    let plan = fixture::plan(&paths);
    model::assert_plan(&plan);
    assert_eq!(
        canonical_before_plan,
        std::fs::read(&paths.project).unwrap()
    );

    fixture::render(&paths);
    let output = paths.rendered.join("preview.mp4");
    media::assert_delivery(&output);
    media::assert_audio_timing(&output);
    media::assert_caption_timing(&output);

    fixture::assert_stale_identity_stops_before_ffmpeg(&paths);
}
