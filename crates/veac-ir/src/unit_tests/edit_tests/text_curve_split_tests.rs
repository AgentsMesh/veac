use super::*;

#[test]
fn split_slices_text_reveal_and_opacity_curves_with_continuous_boundaries() {
    let mut project = sample_project();
    let clip = &mut project.project.sequences[0].tracks[1].clips[0];
    let ClipSource::Caption { style, .. } = &mut clip.source else {
        panic!("caption")
    };
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Grapheme,
        transform: TextUnitTransform::default(),
        reveal: curve("text_reveal"),
        highlight: None,
        opacity: curve("text_opacity"),
        stagger: time(10),
    });
    let edit = batch(
        "op_split_text_curves",
        &project,
        vec![EditOperation::SplitClip {
            clip_id: ItemId::new("itm_caption").unwrap(),
            at: time(150),
            right_clip_id: ItemId::new("itm_caption_right").unwrap(),
            relation_fragments: vec![],
        }],
    );
    let updated = applied(apply_edit_batch(&project, &edit));
    let clips = &updated.project.sequences[0].tracks[1].clips;
    assert_eq!(clips.len(), 2);
    let left = animation(&clips[0]);
    let right = animation(&clips[1]);
    assert_eq!(values(&left.reveal), [0.0, 0.5]);
    assert_eq!(values(&right.reveal), [0.5, 1.0]);
    assert_eq!(values(&left.opacity), [0.0, 0.5]);
    assert_eq!(values(&right.opacity), [0.5, 1.0]);
    let left_ids = ids(&left.reveal);
    let right_ids = ids(&right.reveal);
    assert!(left_ids.iter().all(|id| !right_ids.contains(id)));
}

fn animation(clip: &Clip) -> &TextAnimation {
    match &clip.source {
        ClipSource::Caption { style, .. } => style.animation.as_ref().unwrap(),
        _ => panic!("caption"),
    }
}

fn curve(name: &str) -> Animatable<f64> {
    Animatable::Keyframes {
        keyframes: vec![
            number_key(&format!("kf_{name}_start"), 0, 0.0),
            number_key(&format!("kf_{name}_end"), 300, 1.0),
        ],
    }
}

fn values(value: &Animatable<f64>) -> Vec<f64> {
    value
        .keyframes()
        .unwrap()
        .iter()
        .map(|key| key.value)
        .collect()
}

fn ids(value: &Animatable<f64>) -> Vec<KeyframeId> {
    value
        .keyframes()
        .unwrap()
        .iter()
        .map(|key| key.id.clone())
        .collect()
}
