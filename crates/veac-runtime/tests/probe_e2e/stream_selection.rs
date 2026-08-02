use tempfile::tempdir;
use veac_ir::{StreamChoice, StreamIntent};
use veac_runtime::asset::{probe, probe_with_intent, selected_audio, selected_video};

use super::support::multiple_video_fixture;

#[test]
fn real_probe_honors_default_and_explicit_global_stream_selection() {
    let temp = tempdir().unwrap();
    let media = multiple_video_fixture(temp.path());
    let automatic = probe(&media).unwrap();
    assert_eq!(automatic.selected_video_stream.unwrap().global_index, 1);
    let video = selected_video(&automatic).unwrap().video.as_ref().unwrap();
    assert_eq!((video.width, video.height), (96, 54));

    let explicit = probe_with_intent(
        &media,
        StreamIntent {
            video: StreamChoice::GlobalIndex { global_index: 0 },
            audio: StreamChoice::GlobalIndex { global_index: 2 },
        },
    )
    .unwrap();
    assert_eq!(explicit.selected_video_stream.unwrap().global_index, 0);
    assert_eq!(explicit.selected_video_stream.unwrap().type_index, 0);
    assert_eq!(explicit.selected_audio_stream.unwrap().global_index, 2);
    let video = selected_video(&explicit).unwrap().video.as_ref().unwrap();
    assert_eq!((video.width, video.height), (64, 36));
    let audio = selected_audio(&explicit).unwrap().audio.as_ref().unwrap();
    assert_eq!((audio.sample_rate, audio.channels), (44_100, 1));
}
