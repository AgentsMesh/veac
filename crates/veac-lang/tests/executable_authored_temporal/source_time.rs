use veac_ir::{TemporalClock, TemporalClockOwner};
use veac_lang::program::build_source;

use super::support::MEDIA;

fn source(resource: &str) -> String {
    format!(
        "animate visual-opacity on clip(@media, @main, @visual, @hero-clip) \
         using resource(@media, @{resource}) {{ source_time / 3s }}\n{MEDIA}"
    )
}

#[test]
fn authored_source_time_uses_the_verified_item_clock_owner() {
    let built = build_source(&source("hero")).unwrap();
    let envelope = built.envelope();
    let clip = &envelope.project.sequences[0].tracks[0].clips[0];
    let clock = &envelope.temporal.bindings[0].clocks[0];
    assert_eq!(clock.clock, TemporalClock::SourceTime);
    assert_eq!(
        clock.owner,
        TemporalClockOwner::Item {
            item_id: clip.id.clone()
        }
    );
    assert!(veac_ir::validate(envelope).is_ok());
}

#[test]
fn authored_source_time_rejects_a_non_owner_resource() {
    let error = build_source(&source("other")).unwrap_err();
    assert!(error.as_slice()[0]
        .message
        .contains("EXECUTABLE_TEMPORAL_SOURCE_OWNER"));
}

#[test]
fn source_resource_project_must_match_the_target_project() {
    let source =
        source("hero").replace("resource(@media, @hero)", "resource(@other-project, @hero)");
    let error = build_source(&source).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_TEMPORAL_SOURCE_PROJECT");
}
