use super::*;
use crate::{test_support, ArtifactErrorKind};

#[test]
fn source_recording_requires_the_requested_logical_stream() {
    let mut input = test_support::plan(b"source").inputs.remove(0);
    input.video = None;
    let mut binding = InputBinding::default();

    assert_eq!(
        binding
            .record_source(&input, MediaRole::Video)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::InvalidContract
    );
}
