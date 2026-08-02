use crate::test_support::{linked_project, sample_project, time};

use super::*;

fn fragment(source: &str, right: &str) -> OverwriteFragment {
    OverwriteFragment {
        source_clip_id: ItemId::new(source).unwrap(),
        right_fragment_id: ItemId::new(right).unwrap(),
        relation_fragments: vec![],
    }
}

#[test]
fn fragment_map_accepts_unique_ids_and_rejects_every_ambiguity() {
    let project = sample_project().project;
    let inserted = ItemId::new("itm_inserted").unwrap();
    let values = [fragment("itm_video", "itm_right")];
    let mapped = fragment_map(&project, &values, &inserted).unwrap();
    assert_eq!(
        mapped[&ItemId::new("itm_video").unwrap()]
            .right_fragment_id
            .as_str(),
        "itm_right"
    );

    let failures = [
        vec![fragment("itm_video", "itm_caption")],
        vec![fragment("itm_video", "itm_inserted")],
        vec![
            fragment("itm_video", "itm_right"),
            fragment("itm_caption", "itm_right"),
        ],
        vec![
            fragment("itm_video", "itm_right"),
            fragment("itm_video", "itm_other_right"),
        ],
    ];
    for values in failures {
        let error = fragment_map(&project, &values, &inserted).unwrap_err();
        assert!(error.message.contains("new and unambiguous"));
    }
}

#[test]
fn fragment_sources_must_exactly_match_spanning_clips() {
    let project = sample_project().project;
    let track = &project.sequences[0].tracks[0];
    let exact_fragment = fragment("itm_video", "itm_right");
    let exact = BTreeMap::from([(exact_fragment.source_clip_id.clone(), exact_fragment)]);
    validate_fragment_sources(track, time(100), time(200), &exact).unwrap();
    assert!(validate_fragment_sources(track, time(100), time(200), &BTreeMap::new()).is_err());

    let extra_fragment = fragment("itm_caption", "itm_right");
    let extra = BTreeMap::from([(extra_fragment.source_clip_id.clone(), extra_fragment)]);
    assert!(validate_fragment_sources(track, time(100), time(200), &extra).is_err());
    validate_fragment_sources(track, time(0), time(600), &BTreeMap::new()).unwrap();
}

#[test]
fn target_track_reports_missing_and_locked_targets() {
    let mut project = sample_project().project;
    let sequence = SequenceId::new("seq_main").unwrap();
    let track = TrackId::new("trk_video").unwrap();
    assert_eq!(
        target_track(&mut project, &sequence, &track).unwrap().id,
        track
    );
    assert!(target_track(
        &mut project,
        &SequenceId::new("seq_missing").unwrap(),
        &track,
    )
    .is_err());
    assert!(target_track(
        &mut project,
        &sequence,
        &TrackId::new("trk_missing").unwrap(),
    )
    .is_err());
    project.sequences[0].tracks[0].state.locked = true;
    assert!(target_track(&mut project, &sequence, &track).is_err());
}

#[test]
fn relationship_overlap_checks_only_the_target_interval() {
    let mut grouped = sample_project().project;
    grouped.relations.push(Relation {
        id: RelationId::new("rel_related").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Group {
            members: vec![
                RelationEndpoint::Item {
                    item_id: ItemId::new("itm_video").unwrap(),
                },
                RelationEndpoint::Item {
                    item_id: ItemId::new("itm_caption").unwrap(),
                },
            ],
        },
    });
    let sequence = SequenceId::new("seq_main").unwrap();
    let track = TrackId::new("trk_video").unwrap();
    assert!(reject_related_overlaps(&grouped, &sequence, &track, time(100), time(200)).is_err());
    reject_related_overlaps(&grouped, &sequence, &track, time(600), time(700)).unwrap();

    let linked = linked_project().project;
    assert!(reject_related_overlaps(&linked, &sequence, &track, time(100), time(200)).is_err());
    assert!(reject_related_overlaps(
        &linked,
        &SequenceId::new("seq_missing").unwrap(),
        &track,
        time(0),
        time(1),
    )
    .is_err());
    assert!(reject_related_overlaps(
        &linked,
        &sequence,
        &TrackId::new("trk_missing").unwrap(),
        time(0),
        time(1),
    )
    .is_err());
}
