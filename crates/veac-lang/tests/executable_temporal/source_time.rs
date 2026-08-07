use veac_ir::{MaterialId, TemporalClock, TemporalClockOwner};
use veac_lang::program::expression::CoreTemporalInputIdentity;
use veac_lang::program::ClipTemporalProperty;

use super::support::{self, MEDIA_SOURCE};

fn source_leaf(material: MaterialId) -> veac_lang::program::ExecutableTemporalLeaf {
    let ids = support::ids(MEDIA_SOURCE);
    let expression = support::compile(
        "source_time / 1s",
        [(
            "source_time",
            CoreTemporalInputIdentity::SourceTime {
                source_id: material,
            },
        )],
    );
    support::leaf(
        ids.items[0].clone(),
        ClipTemporalProperty::VisualOpacity,
        expression,
        "source_time",
    )
}

#[test]
fn source_time_uses_item_clock_owner_after_material_identity_is_verified() {
    let ids = support::ids(MEDIA_SOURCE);
    let envelope = support::execute(MEDIA_SOURCE, &[source_leaf(ids.material.unwrap())]);
    let clock = &envelope.temporal.bindings[0].clocks[0];
    assert_eq!(clock.clock, TemporalClock::SourceTime);
    assert_eq!(
        clock.owner,
        TemporalClockOwner::Item {
            item_id: ids.items[0].clone(),
        }
    );
    assert!(veac_ir::validate(&envelope).is_ok());
}

#[test]
fn source_time_rejects_a_material_other_than_the_target_clip_source() {
    let leaf = source_leaf(MaterialId::new("med_wrong").unwrap());
    let diagnostic = support::failure(MEDIA_SOURCE, &[leaf]);
    assert_eq!(diagnostic.code, "PROGRAM_EXECUTABLE_LOWER");
    assert!(diagnostic
        .message
        .contains("EXECUTABLE_TEMPORAL_SOURCE_OWNER"));
}
