//! Real ffprobe integration tests. Fixtures are generated from lavfi at test time.

#[path = "probe_e2e/attached_picture.rs"]
mod attached_picture;
#[path = "probe_e2e/indirect_inputs.rs"]
mod indirect_inputs;
#[path = "probe_e2e/stream_selection.rs"]
mod stream_selection;
#[path = "probe_e2e/support.rs"]
mod support;
