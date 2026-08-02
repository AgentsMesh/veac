use super::*;

fn speaker(project: &mut ProjectEnvelope) -> &mut Option<String> {
    let ClipSource::Caption { speaker, .. } =
        &mut project.project.sequences[0].tracks[1].clips[0].source
    else {
        panic!("caption fixture")
    };
    speaker
}

#[test]
fn caption_speaker_is_typed_and_validated() {
    let mut project = sample_project();
    *speaker(&mut project) = Some("Narrator".to_owned());
    validate(&project).unwrap();

    for invalid in [" ".to_owned(), "Alice\nBob".to_owned(), "x".repeat(257)] {
        *speaker(&mut project) = Some(invalid);
        assert_code(&validation_codes(&project), "CAPTION_SPEAKER");
    }
}
