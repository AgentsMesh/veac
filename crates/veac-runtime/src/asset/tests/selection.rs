use serde_json::json;
use veac_ir::{ProbedStreamType, StreamChoice};

use super::*;

fn selection_output() -> Value {
    json!({ "format": { "format_name": "mov,mp4,m4a,3gp,3g2,mj2" }, "streams": [
        {
            "index": 0, "codec_type": "video", "codec_name": "mjpeg",
            "avg_frame_rate": "0/0", "r_frame_rate": "0/0",
            "width": 600, "height": 600, "pix_fmt": "yuvj420p",
            "sample_aspect_ratio": "N/A",
            "disposition": { "default": 1, "attached_pic": 1 }
        },
        {
            "index": 1, "codec_type": "video", "codec_name": "preview",
            "time_base": "1/30000", "avg_frame_rate": "30/1", "r_frame_rate": "30/1",
            "width": 640, "height": 360, "pix_fmt": "yuv420p",
            "sample_aspect_ratio": "4:3",
            "tags": { "rotate": "180" }, "disposition": { "default": 0 }
        },
        {
            "index": 2, "codec_type": "video", "codec_name": "thumbnail",
            "avg_frame_rate": "0/0", "r_frame_rate": "0/0",
            "width": 320, "height": 180, "pix_fmt": "yuv420p",
            "disposition": { "timed_thumbnails": 1 }
        },
        {
            "index": 3, "codec_type": "video", "codec_name": "main",
            "time_base": "1/24000", "avg_frame_rate": "24/1", "r_frame_rate": "24/1",
            "width": 1280, "height": 720, "pix_fmt": "yuv420p",
            "disposition": { "default": 1 }
        },
        {
            "index": 4, "codec_type": "audio", "codec_name": "aac",
            "time_base": "1/44100",
            "sample_rate": "44100", "channels": 1, "channel_layout": "mono"
        }
    ] })
}

#[test]
fn auto_selection_prefers_default_and_excludes_cover_art_and_thumbnails() {
    let snapshot = parse(&selection_output(), auto_stream_intent()).unwrap();
    assert_eq!(snapshot.selected_video_stream.unwrap().global_index, 3);
    assert_eq!(snapshot.selected_video_stream.unwrap().type_index, 3);
    assert_eq!(snapshot.selected_audio_stream.unwrap().global_index, 4);
    assert!(snapshot.streams[0].disposition.attached_picture);
    assert!(snapshot.streams[2].disposition.timed_thumbnail);
}

#[test]
fn explicit_and_disabled_selection_follow_stream_intent() {
    let explicit = parse(
        &selection_output(),
        intent(
            StreamChoice::GlobalIndex { global_index: 1 },
            StreamChoice::Disabled,
        ),
    )
    .unwrap();
    assert_eq!(explicit.selected_video_stream.unwrap().global_index, 1);
    assert!(explicit.selected_audio_stream.is_none());
    let video = selected_video(&explicit).unwrap().video.as_ref().unwrap();
    assert_eq!(video.rotation_degrees, 180);
    assert_eq!(
        (
            video.sample_aspect_ratio.numerator,
            video.sample_aspect_ratio.denominator
        ),
        (4, 3)
    );
}

#[test]
fn explicit_selection_rejects_wrong_or_non_playable_streams() {
    for global_index in [0, 2, 4, 99] {
        let error = parse(
            &selection_output(),
            intent(
                StreamChoice::GlobalIndex { global_index },
                StreamChoice::Auto,
            ),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            ProbeError::StreamSelection {
                media_type: ProbedStreamType::Video,
                global_index: actual,
            } if actual == global_index
        ));
    }
}

#[test]
fn auto_selection_allows_an_absent_media_type() {
    let value = json!({ "format": { "format_name": "data" }, "streams": [{
        "index": 0, "codec_type": "data", "codec_name": "bin_data"
    }] });
    let snapshot = parse(&value, auto_stream_intent()).unwrap();
    assert!(snapshot.selected_video_stream.is_none());
    assert!(snapshot.selected_audio_stream.is_none());
}

#[test]
fn auto_selection_keeps_the_first_playable_stream_without_a_default() {
    let mut value = selection_output();
    value["streams"][3]["disposition"]["default"] = json!(0);
    let snapshot = parse(&value, auto_stream_intent()).unwrap();
    assert_eq!(snapshot.selected_video_stream.unwrap().global_index, 1);
}
