use veac_ir::Animatable;
use veac_lang::program::expression::CoreTemporalInputIdentity;
use veac_lang::program::ClipTemporalProperty as Property;

use super::sink_support;
use super::support;

fn leaf(
    item: &veac_ir::ItemId,
    property: Property,
    source: &str,
    name: &str,
) -> veac_lang::program::ExecutableTemporalLeaf {
    let expression = support::compile(
        source,
        [(
            "progress",
            CoreTemporalInputIdentity::Progress {
                item_id: item.clone(),
            },
        )],
    );
    support::leaf(item.clone(), property, expression, name)
}

#[test]
fn every_published_clip_sink_receives_a_typed_canonical_binding() {
    let (visual_id, audio_id) = sink_support::items();
    let leaves = [
        leaf(
            &visual_id,
            Property::VisualPosition,
            "point(progress * 10px, progress * 20px)",
            "position",
        ),
        leaf(
            &visual_id,
            Property::VisualScale,
            "vector(progress, progress)",
            "scale",
        ),
        leaf(
            &visual_id,
            Property::VisualRotation,
            "progress * 90deg",
            "rotation",
        ),
        leaf(
            &visual_id,
            Property::VisualCrop,
            "rect(progress, 0.0, 1.0, 1.0)",
            "crop",
        ),
        leaf(
            &visual_id,
            Property::VisualOpacity,
            "progress",
            "opacity_matrix",
        ),
        leaf(&audio_id, Property::AudioGain, "progress", "gain"),
        leaf(&audio_id, Property::AudioPan, "progress * 2.0 - 1.0", "pan"),
    ];
    let envelope = support::execute(sink_support::SOURCE, &leaves);
    let sequence = &envelope.project.sequences[0];
    let visual = sequence.tracks[0].clips[0].visual.as_ref().unwrap();
    for value in [
        visual.transform.position.binding_id(),
        visual.transform.scale.binding_id(),
        visual.transform.rotation_degrees.binding_id(),
        visual.transform.crop.as_ref().unwrap().binding_id(),
        visual.opacity.binding_id(),
    ] {
        assert!(value.is_some());
    }
    let audio = sequence.tracks[1].clips[0].audio.as_ref().unwrap();
    assert!(matches!(audio.gain, Animatable::Binding { .. }));
    assert!(matches!(audio.pan, Animatable::Binding { .. }));
    assert_eq!(envelope.temporal.bindings.len(), 7);
    assert!(veac_ir::validate(&envelope).is_ok());
}

#[test]
fn temporal_crop_cannot_create_an_absent_optional_static_leaf() {
    let ids = support::ids(support::SOLID_SOURCE);
    let leaf = leaf(
        &ids.items[0],
        Property::VisualCrop,
        "rect(progress, 0.0, 1.0, 1.0)",
        "absent_crop",
    );
    let diagnostic = support::failure(support::SOLID_SOURCE, &[leaf]);
    assert!(diagnostic
        .message
        .contains("EXECUTABLE_TEMPORAL_OPTIONAL_SINK"));
}
