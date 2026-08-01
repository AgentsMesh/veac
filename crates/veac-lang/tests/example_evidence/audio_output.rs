use crate::support::lower_example;
use veac_ir::{AudioCodec, DeliverableKind, OutputFormat};

#[test]
fn audio_processing_preview_encodes_the_processed_mix() {
    let envelope = lower_example("audio-processing/main.veac");
    let output = envelope
        .project
        .render_configs
        .iter()
        .flat_map(|config| &config.deliverables)
        .find_map(|deliverable| match &deliverable.kind {
            DeliverableKind::Video(video) => Some(video),
            _ => None,
        })
        .expect("preview video output");
    let audio = output
        .audio
        .as_ref()
        .expect("preview should retain the processed audio mix");

    assert_eq!(output.container, OutputFormat::Mp4);
    assert_eq!(audio.codec, AudioCodec::Aac);
    assert_eq!(audio.sample_rate, 48_000);
    assert_eq!(audio.channels, 2);
}
