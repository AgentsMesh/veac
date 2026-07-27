use tempfile::tempdir;
use veac_ir::{ProbedStreamType, StreamChoice, StreamIntent};
use veac_runtime::asset::{probe, probe_with_intent, selected_audio, selected_video, ProbeError};

use super::support::attached_picture_fixture;

#[test]
fn real_probe_excludes_attached_picture_from_playable_video_selection() {
    let temp = tempdir().unwrap();
    let media = attached_picture_fixture(temp.path());
    let snapshot = probe(&media).unwrap();

    assert!(snapshot.engine.starts_with("ffprobe version "));
    assert_eq!(snapshot.observed_identity.digest.len(), 64);
    assert!(snapshot.container_duration.is_some());
    assert_eq!(snapshot.selected_video_stream.unwrap().global_index, 0);
    assert_eq!(snapshot.selected_audio_stream.unwrap().global_index, 1);
    let playable = selected_video(&snapshot).unwrap();
    let video = playable.video.as_ref().unwrap();
    assert_eq!((video.width, video.height), (64, 36));
    assert_eq!(
        selected_audio(&snapshot)
            .unwrap()
            .audio
            .as_ref()
            .unwrap()
            .sample_rate,
        48_000
    );

    let cover = snapshot
        .streams
        .iter()
        .find(|stream| stream.disposition.attached_picture)
        .expect("attached picture stream");
    assert_eq!(cover.media_type, ProbedStreamType::Video);
    assert_eq!(cover.global_index, 2);
    assert_eq!(cover.type_index, 1);
    assert_ne!(
        snapshot.selected_video_stream.unwrap().global_index,
        cover.global_index
    );

    let error = probe_with_intent(
        &media,
        StreamIntent {
            video: StreamChoice::GlobalIndex {
                global_index: cover.global_index,
            },
            audio: StreamChoice::Auto,
        },
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ProbeError::StreamSelection {
            global_index: 2,
            ..
        }
    ));
}
