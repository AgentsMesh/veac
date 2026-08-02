use crate::edit::ChangeSet;
use crate::test_support::{sample_project, time};

use super::*;

#[test]
fn ripple_trim_rejects_connected_clips_sharing_a_track() {
    let mut project = sample_project().project;
    let track = &mut project.sequences[0].tracks[0];
    track.placement_mode = PlacementMode::Free;
    let mut second = track.clips[0].clone();
    second.id = ItemId::new("itm_second").unwrap();
    second.record_range.start = time(600);
    track.clips.push(second);
    add_group(&mut project);
    let before = project.clone();
    let error = trim(
        &mut project,
        &ItemId::new("itm_video").unwrap(),
        TrimEdge::Out,
        time(-1),
        true,
        &mut ChangeSet::new(),
    )
    .unwrap_err();
    assert!(error
        .message
        .contains("multiple grouped clips on one track"));
    assert_eq!(project, before);
}

#[test]
fn non_ripple_trim_allows_connected_clips_on_the_same_track() {
    let mut project = sample_project().project;
    let track = &mut project.sequences[0].tracks[0];
    track.placement_mode = PlacementMode::Free;
    let mut second = track.clips[0].clone();
    second.id = ItemId::new("itm_second").unwrap();
    second.record_range.start = time(600);
    track.clips.push(second);
    add_group(&mut project);
    trim(
        &mut project,
        &ItemId::new("itm_video").unwrap(),
        TrimEdge::Out,
        time(-1),
        false,
        &mut ChangeSet::new(),
    )
    .unwrap();
    assert!(project.sequences[0].tracks[0]
        .clips
        .iter()
        .all(|clip| clip.record_range.duration == time(599)));
}

fn add_group(project: &mut Project) {
    project.relations.push(Relation {
        id: RelationId::new("rel_same_track").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Group {
            members: vec![
                RelationEndpoint::Item {
                    item_id: ItemId::new("itm_video").unwrap(),
                },
                RelationEndpoint::Item {
                    item_id: ItemId::new("itm_second").unwrap(),
                },
            ],
        },
    });
}
