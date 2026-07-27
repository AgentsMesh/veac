use super::*;

use super::apply_target_tests::base_apply;

#[test]
fn canonical_json_rejects_legacy_adjustment_sources() {
    let mut value = serde_json::to_value(sample_project()).unwrap();
    value["project"]["sequences"][0]["tracks"][0]["clips"][0]["source"] = serde_json::json!({
        "type": "adjustment_layer",
        "target_track_id": "trk_video"
    });
    assert!(serde_json::from_value::<ProjectEnvelope>(value).is_err());
}

#[test]
fn canonical_json_requires_first_class_applies() {
    let mut value = serde_json::to_value(sample_project()).unwrap();
    value["project"]["sequences"][0]
        .as_object_mut()
        .unwrap()
        .remove("applies");
    assert!(serde_json::from_value::<ProjectEnvelope>(value).is_err());
}

#[test]
fn canonical_json_rejects_unknown_apply_fields_and_targets() {
    let mut project = sample_project();
    project.project.sequences[0].applies.push(base_apply(
        "apl_json",
        "aps_json",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    ));
    let mut unknown = serde_json::to_value(&project).unwrap();
    unknown["project"]["sequences"][0]["applies"][0]
        .as_object_mut()
        .unwrap()
        .insert("target_track_id".into(), serde_json::json!("trk_video"));
    assert!(serde_json::from_value::<ProjectEnvelope>(unknown).is_err());

    let mut target = serde_json::to_value(project).unwrap();
    target["project"]["sequences"][0]["applies"][0]["target"] = serde_json::json!({
        "type": "track",
        "track_id": "trk_video"
    });
    assert!(serde_json::from_value::<ProjectEnvelope>(target).is_err());
}
