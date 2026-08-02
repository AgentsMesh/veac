use super::*;
use crate::test_support::{sample_project, time};

fn apply(id: &str, target: ApplyTarget) -> Apply {
    Apply {
        id: ApplyId::new(id).expect("apply id"),
        enabled: true,
        record_range: TimeRange::new(time(0), time(600)).expect("apply range"),
        target,
        stages: Vec::new(),
        mix: ApplyMix::default(),
    }
}

#[test]
fn guards_all_apply_target_scopes_and_reference_mutations() {
    let mut envelope = sample_project();
    let track_id = envelope.project.sequences[0].tracks[0].id.clone();
    let item_id = envelope.project.sequences[0].tracks[0].clips[0].id.clone();
    let missing_item = ItemId::new("itm_missing").expect("missing item id");
    let missing_track = TrackId::new("trk_missing").expect("missing track id");

    let item_apply = apply(
        "apl_items",
        ApplyTarget::ItemSet {
            item_ids: vec![item_id.clone()],
        },
    );
    let layer_apply = apply(
        "apl_layer",
        ApplyTarget::Layer {
            track_id: track_id.clone(),
        },
    );
    let band_apply = apply(
        "apl_band",
        ApplyTarget::CompositeBand {
            from_track_id: track_id.clone(),
            through_track_id: track_id.clone(),
        },
    );
    envelope.project.sequences[0].applies =
        vec![item_apply.clone(), layer_apply.clone(), band_apply.clone()];

    assert_eq!(
        target_track_ids(&envelope.project.sequences[0], &item_apply.target).expect("item target"),
        BTreeSet::from([track_id.clone()])
    );
    assert_eq!(
        target_track_ids(&envelope.project.sequences[0], &band_apply.target).expect("band target"),
        BTreeSet::from([track_id.clone()])
    );
    assert_eq!(
        item_track(&envelope.project.sequences[0], &item_id)
            .expect("item track")
            .id,
        track_id
    );
    assert!(item_track(&envelope.project.sequences[0], &missing_item).is_err());
    assert!(target_track_ids(
        &envelope.project.sequences[0],
        &ApplyTarget::Layer {
            track_id: missing_track,
        },
    )
    .is_err());
    assert!(target_track_ids(
        &envelope.project.sequences[0],
        &ApplyTarget::ItemSet {
            item_ids: vec![missing_item.clone()],
        },
    )
    .is_err());
    assert!(target_track_ids(
        &envelope.project.sequences[0],
        &ApplyTarget::CompositeBand {
            from_track_id: TrackId::new("trk_missing_from").expect("track id"),
            through_track_id: track_id.clone(),
        },
    )
    .is_err());
    assert!(target_track_ids(
        &envelope.project.sequences[0],
        &ApplyTarget::CompositeBand {
            from_track_id: track_id.clone(),
            through_track_id: TrackId::new("trk_missing_through").expect("track id"),
        },
    )
    .is_err());

    assert!(ensure_items_removable(
        &envelope.project.sequences[0],
        &BTreeSet::from([item_id.clone()]),
    )
    .is_err());
    assert!(ensure_track_removable(&envelope.project, &track_id).is_err());
    let mut second_track = envelope.project.sequences[0].tracks[0].clone();
    second_track.id = TrackId::new("trk_second").expect("track id");
    second_track.order = 1;
    second_track.clips[0].id = ItemId::new("itm_second").expect("item id");
    second_track.state.locked = true;
    envelope.project.sequences[0]
        .tracks
        .push(second_track.clone());
    envelope.project.sequences[0].applies[2].target = ApplyTarget::CompositeBand {
        from_track_id: track_id.clone(),
        through_track_id: second_track.id,
    };
    assert!(ensure_order_change_unlocked(&envelope.project, &track_id, 2).is_err());
    assert_eq!(
        require_track(&envelope.project.sequences[0], &track_id)
            .expect("target")
            .id,
        track_id
    );
    assert!(require_track(
        &envelope.project.sequences[0],
        &TrackId::new("trk_missing_required").expect("track id"),
    )
    .is_err());

    envelope.project.sequences[0].tracks[0].state.locked = true;
    assert!(ensure_target_unlocked(
        &envelope.project.sequences[0],
        &layer_apply.target,
        &BTreeSet::new(),
    )
    .is_err());
}
