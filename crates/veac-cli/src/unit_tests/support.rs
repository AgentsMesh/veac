use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::PathBuf;

use tempfile::TempDir;
use veac_artifact::ContentDigest;
use veac_ir::{HashAlgorithm, MediaIdentity, MediaProbeSnapshot, StreamIntent};
mod capability;
mod fake_environment;
mod project;
mod reachability;
mod source;
mod template;

use capability::successful_filters;
pub(crate) use project::{add_second_video, canonical_project, pin_first_material, render};
pub(crate) use reachability::{
    accessed_names, assert_exact_inputs, assert_project_inputs, write_project,
};
pub(crate) use source::EXECUTABLE_SOURCE;
pub(crate) use source::GENERATED_SOURCE;
pub(crate) use source::MEDIA_SOURCE;
pub(crate) use template::{MEDIA_TEMPLATE_SOURCE, TEXT_TEMPLATE_SOURCE};

pub(super) fn identity(byte: u8) -> MediaIdentity {
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: format!("{byte:02x}").repeat(32),
    }
}

pub(super) fn snapshot(intent: StreamIntent, observed: MediaIdentity) -> MediaProbeSnapshot {
    let json = r#"{
      "format":{"format_name":"mov,mp4,m4a,3gp,3g2,mj2","duration":"1.000000",
        "tags":{"major_brand":"isom"}},
      "streams":[{"index":0,"codec_type":"video","codec_name":"h264",
        "time_base":"1/600","avg_frame_rate":"10/1","r_frame_rate":"10/1",
        "start_time":"0","width":32,"height":24,"pix_fmt":"yuv420p",
        "profile":"High","level":31,"sample_aspect_ratio":"1:1","duration":"1.000000",
        "disposition":{"default":1,"attached_pic":0,"timed_thumbnails":0}},
        {"index":1,"codec_type":"audio","codec_name":"aac","time_base":"1/48000",
        "start_time":"0","duration":"1.000000","sample_rate":"48000","channels":2,
        "channel_layout":"stereo","disposition":{"default":1,"attached_pic":0,"timed_thumbnails":0}}]
    }"#;
    let mut snapshot = veac_runtime::asset::parse_ffprobe_json(
        json,
        observed,
        intent,
        veac_runtime::asset::FIXTURE_PROBE_ENGINE,
    )
    .unwrap();
    let selected = [
        snapshot.selected_video_stream,
        snapshot.selected_audio_stream,
    ];
    snapshot.streams.retain(|stream| {
        selected
            .iter()
            .flatten()
            .any(|selection| selection.global_index == stream.global_index)
    });
    snapshot
}

pub(crate) struct FakeEnvironment {
    pub observed: MediaIdentity,
    pub identity_paths: RefCell<Vec<PathBuf>>,
    pub probe_paths: RefCell<Vec<PathBuf>>,
    pub probe_intents: RefCell<Vec<StreamIntent>>,
    pub executed: RefCell<Vec<Vec<String>>>,
    pub consumed_inputs: RefCell<Vec<Vec<Vec<u8>>>>,
    pub fail_probe: bool,
    pub fail_execute: bool,
    pub fail_version: bool,
    pub encoders: BTreeSet<String>,
    pub muxers: BTreeSet<String>,
    pub decoders: BTreeSet<String>,
    pub demuxers: BTreeSet<String>,
    pub filters: BTreeSet<String>,
    pub hardware_backends: BTreeSet<String>,
    pub hardware_devices: BTreeSet<String>,
    pub fingerprint_configuration: ContentDigest,
}

impl FakeEnvironment {
    pub(crate) fn success() -> Self {
        Self {
            observed: identity(0x11),
            identity_paths: RefCell::new(Vec::new()),
            probe_paths: RefCell::new(Vec::new()),
            probe_intents: RefCell::new(Vec::new()),
            executed: RefCell::new(Vec::new()),
            consumed_inputs: RefCell::new(Vec::new()),
            fail_probe: false,
            fail_execute: false,
            fail_version: false,
            encoders: ["libx264", "libx265", "png", "prores_ks"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            muxers: ["image2", "mov", "mp4", "mxf", "null"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            decoders: ["h264", "aac"].into_iter().map(str::to_owned).collect(),
            demuxers: ["mov", "wav"].into_iter().map(str::to_owned).collect(),
            filters: successful_filters(),
            hardware_backends: BTreeSet::new(),
            hardware_devices: BTreeSet::new(),
            fingerprint_configuration: ContentDigest::sha256("fixture configuration"),
        }
    }
}

pub(super) fn source_file(temp: &TempDir, source: &str) -> PathBuf {
    let path = temp.path().join("main.veac");
    std::fs::write(&path, source).unwrap();
    path
}
