use super::*;
use crate::unit_tests::emitter_tests::support::{fixture, resolved};

#[test]
fn unsupported_sidechain_diagnostic_keeps_clip_identity() {
    let plan = resolved(&fixture());
    let clip = &plan.sequences[0].tracks[0].clips[0];
    let error = unsupported(clip, "sidechain contract");
    let diagnostic = &error.diagnostics()[0];
    assert_eq!(diagnostic.code, "AUDIO_PROCESSING_UNSUPPORTED");
    assert_eq!(diagnostic.object_id.as_deref(), Some(clip.id.as_str()));
}
