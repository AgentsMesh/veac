use veac_ir::{StreamChoice, StreamIntent};

mod display;
mod error;
mod ffprobe;
mod identity;
mod probe_json;
mod selection;
mod time;

pub use display::{selected_audio, selected_video, ProbeDisplay};
pub use error::ProbeError;
pub use ffprobe::{probe, probe_with_intent, SystemFfprobe};
pub use identity::sha256_identity;
pub use probe_json::{
    parse_ffprobe_json, FIXTURE_PROBE_ENGINE, PROBE_SCHEMA_VERSION, STREAM_SELECTION_POLICY,
};

/// The default selection intent used by [`probe`].
pub fn auto_stream_intent() -> StreamIntent {
    StreamIntent {
        video: StreamChoice::Auto,
        audio: StreamChoice::Auto,
    }
}

#[cfg(test)]
#[path = "asset/tests.rs"]
mod tests;
