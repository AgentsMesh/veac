use super::*;

#[test]
fn whole_component_replacement_reports_every_old_and_new_curve_key() {
    let project = sample_project();
    let video_id = ItemId::new("itm_video").unwrap();
    let caption_id = ItemId::new("itm_caption").unwrap();
    let mut visual = project.project.sequences[0].tracks[0].clips[0]
        .visual
        .clone()
        .unwrap();
    visual.transform.position = Animatable::Keyframes {
        keyframes: vec![
            point_key("kf_tree_pos_a", 0),
            point_key("kf_tree_pos_b", 60),
        ],
    };
    visual.transform.scale = vec_curve("kf_tree_scale", 1.0, 1.2);
    visual.transform.rotation_degrees = number_curve("kf_tree_rot", 0.0, 10.0);
    visual.opacity = number_curve("kf_tree_opacity", 0.2, 0.8);
    visual.masks[0].position = vec_curve("kf_tree_mask_pos", 0.4, 0.6);
    visual.masks[0].scale = vec_curve("kf_tree_mask_scale", 0.8, 1.0);
    visual.masks[0].rotation_degrees = number_curve("kf_tree_mask_rot", 0.0, 5.0);
    visual.masks[0].feather_pixels = number_curve("kf_tree_mask_feather", 1.0, 4.0);
    visual.masks[0].expansion_pixels = number_curve("kf_tree_mask_expand", 0.0, 2.0);

    let mut audio = project.project.sequences[0].tracks[0].clips[0]
        .audio
        .clone()
        .unwrap();
    audio.gain = number_curve("kf_tree_gain", 0.5, 1.0);
    audio.pan = number_curve("kf_tree_pan", -0.5, 0.5);
    let animation = TextAnimation {
        granularity: TextGranularity::Word,
        transform: TextUnitTransform::default(),
        reveal: number_curve("kf_tree_reveal", 0.0, 1.0),
        highlight: None,
        opacity: number_curve("kf_tree_text_opacity", 0.0, 1.0),
        stagger: time(10),
    };
    let operations = vec![
        EditOperation::SetVisual {
            clip_id: video_id.clone(),
            visual: Some(visual),
        },
        EditOperation::SetAudio {
            clip_id: video_id.clone(),
            audio: Some(audio),
        },
        EditOperation::SetTextProperty {
            clip_id: caption_id.clone(),
            property: TextProperty::Animation(Some(animation)),
        },
    ];
    let (updated, changed) = outcome(apply_edit_batch(
        &project,
        &batch("op_changed_tree_branches", &project, operations),
    ));
    assert!(changed.contains(&ChangedObjectId::Item { id: video_id }));
    assert!(changed.contains(&ChangedObjectId::Item { id: caption_id }));
    for id in expected_keys() {
        assert!(
            changed.contains(&ChangedObjectId::Keyframe {
                id: KeyframeId::new(id).unwrap(),
            }),
            "missing changed key {id}"
        );
    }

    let removals = vec![
        EditOperation::SetVisual {
            clip_id: ItemId::new("itm_video").unwrap(),
            visual: None,
        },
        EditOperation::SetAudio {
            clip_id: ItemId::new("itm_video").unwrap(),
            audio: None,
        },
        EditOperation::SetTextProperty {
            clip_id: ItemId::new("itm_caption").unwrap(),
            property: TextProperty::Animation(None),
        },
    ];
    let (_, removed) = outcome(apply_edit_batch(
        &updated,
        &batch("op_remove_changed_tree", &updated, removals),
    ));
    for id in expected_keys()
        .into_iter()
        .filter(|id| id.starts_with("kf_tree"))
    {
        assert!(removed.contains(&ChangedObjectId::Keyframe {
            id: KeyframeId::new(id).unwrap(),
        }));
    }
}

fn number_curve(prefix: &str, first: f64, second: f64) -> Animatable<f64> {
    Animatable::Keyframes {
        keyframes: vec![
            number_key(&format!("{prefix}_a"), 0, first),
            number_key(&format!("{prefix}_b"), 60, second),
        ],
    }
}

fn vec_curve(prefix: &str, first: f64, second: f64) -> Animatable<Vec2> {
    Animatable::Keyframes {
        keyframes: vec![
            vec2_key(&format!("{prefix}_a"), 0, first),
            vec2_key(&format!("{prefix}_b"), 60, second),
        ],
    }
}

fn expected_keys() -> Vec<&'static str> {
    vec![
        "kf_opacity_start",
        "kf_opacity_end",
        "kf_tree_pos_a",
        "kf_tree_pos_b",
        "kf_tree_scale_a",
        "kf_tree_scale_b",
        "kf_tree_rot_a",
        "kf_tree_rot_b",
        "kf_tree_opacity_a",
        "kf_tree_opacity_b",
        "kf_tree_mask_pos_a",
        "kf_tree_mask_pos_b",
        "kf_tree_mask_scale_a",
        "kf_tree_mask_scale_b",
        "kf_tree_mask_rot_a",
        "kf_tree_mask_rot_b",
        "kf_tree_mask_feather_a",
        "kf_tree_mask_feather_b",
        "kf_tree_mask_expand_a",
        "kf_tree_mask_expand_b",
        "kf_tree_gain_a",
        "kf_tree_gain_b",
        "kf_tree_pan_a",
        "kf_tree_pan_b",
        "kf_tree_reveal_a",
        "kf_tree_reveal_b",
        "kf_tree_text_opacity_a",
        "kf_tree_text_opacity_b",
    ]
}

fn outcome(value: EditOutcome) -> (ProjectEnvelope, Vec<ChangedObjectId>) {
    match value {
        EditOutcome::Applied {
            project,
            changed_objects,
            ..
        } => (project, changed_objects),
        other => panic!("expected applied, got {other:?}"),
    }
}
