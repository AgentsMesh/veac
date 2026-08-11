use serde_json::{json, Value};
use veac_ir::{HashAlgorithm, MediaIdentity, MediaProbeSnapshot, StreamChoice, StreamIntent};

use super::*;

#[path = "tests/errors.rs"]
mod errors;
#[path = "tests/format_errors.rs"]
mod format_errors;
#[path = "tests/identity.rs"]
mod identity;
#[path = "tests/limits.rs"]
mod limits;
#[path = "tests/parsing.rs"]
mod parsing;
#[path = "tests/probe_cadence.rs"]
mod probe_cadence;
#[path = "tests/probe_limits.rs"]
mod probe_limits;
#[path = "tests/probe_process.rs"]
mod probe_process;
#[path = "tests/selection.rs"]
mod selection;
#[path = "tests/video_metadata.rs"]
mod video_metadata;

pub(super) fn identity() -> MediaIdentity {
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: "ab".repeat(32),
    }
}

pub(super) fn intent(video: StreamChoice, audio: StreamChoice) -> StreamIntent {
    StreamIntent { video, audio }
}

pub(super) fn parse(value: &Value, intent: StreamIntent) -> Result<MediaProbeSnapshot, ProbeError> {
    parse_ffprobe_json(&value.to_string(), identity(), intent, FIXTURE_PROBE_ENGINE)
}

pub(super) fn complete_output() -> Value {
    json!({
        "format": {
            "format_name": "mov,mp4,m4a,3gp,3g2,mj2",
            "duration": "12.500000",
            "tags": { "major_brand": "isom" }
        },
        "streams": [
            {
                "index": 4, "codec_type": "audio", "codec_name": "aac",
                "time_base": "1/48000",
                "start_time": "0.000000", "duration": "12.500000",
                "sample_rate": "48000", "channels": 2, "channel_layout": "stereo",
                "disposition": { "default": 1, "attached_pic": 0, "timed_thumbnails": 0 }
            },
            {
                "index": 2, "codec_type": "video", "codec_name": "h264",
                "time_base": "1/12288", "avg_frame_rate": "24/1", "r_frame_rate": "24/1",
                "start_time": "0.250000", "duration": "12.000000",
                "width": 1920, "height": 1080, "pix_fmt": "yuv420p",
                "profile": "High", "level": 40, "sample_aspect_ratio": "1:1",
                "side_data_list": [{ "rotation": -90 }],
                "disposition": { "default": 1, "attached_pic": 0, "timed_thumbnails": 0 }
            },
            {
                "index": 6, "codec_type": "subtitle", "codec_name": "subrip",
                "time_base": "1/1000",
                "start_time": "0", "duration": "10",
                "disposition": { "default": 0, "attached_pic": 0, "timed_thumbnails": 0 }
            }
        ]
    })
}
