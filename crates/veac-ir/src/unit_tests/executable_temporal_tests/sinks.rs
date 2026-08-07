use crate::{
    test_support::{sample_project, time},
    *,
};

use super::support::{add_sequence_clock, bind, binding, codes};

#[test]
fn every_canonical_animation_sink_accepts_its_exact_temporal_type() {
    let mut project = sample_project();
    let scalar = bind(&mut project, TemporalType::Scalar, "scalar");
    let angle = bind(&mut project, TemporalType::Angle, "angle");
    let vector = bind(&mut project, TemporalType::Vec2, "vector");
    let point = bind(&mut project, TemporalType::Point, "point");
    let rect = bind(&mut project, TemporalType::Rect, "rect");

    let video = &mut project.project.sequences[0].tracks[0].clips[0];
    let visual = video.visual.as_mut().unwrap();
    visual.transform.position = binding(&point);
    visual.transform.scale = binding(&vector);
    visual.transform.rotation_degrees = binding(&angle);
    visual.transform.crop = Some(binding(&rect));
    visual.opacity = binding(&scalar);
    bind_mask(&mut visual.masks[0], &scalar, &angle, &vector);
    let audio = video.audio.as_mut().unwrap();
    audio.gain = binding(&scalar);
    audio.pan = binding(&scalar);
    *video.effects[0]
        .effect
        .curve_mut(EffectParameter::Brightness)
        .unwrap() = binding(&scalar);

    let caption = &mut project.project.sequences[0].tracks[1].clips[0];
    let style = match &mut caption.source {
        ClipSource::Caption { style, .. } => style,
        _ => unreachable!(),
    };
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Word,
        transform: TextUnitTransform {
            position_offset: binding(&point),
            scale: binding(&vector),
            rotation_degrees: binding(&angle),
        },
        reveal: binding(&scalar),
        highlight: Some(TextHighlightAnimation {
            fill: Color {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 255,
            },
            progress: binding(&scalar),
        }),
        opacity: binding(&scalar),
        stagger: time(0),
    });

    let mut apply = apply_with(binding(&scalar));
    apply.mix.masks.push(bound_mask(&scalar, &angle, &vector));
    apply.stages.push(ApplyStage {
        id: ApplyStageId::new("aps_effect").unwrap(),
        enabled: true,
        active_range: None,
        operation: ApplyOperation::Effect {
            effect: EffectInstance {
                id: EffectId::new("fx_apply_blur").unwrap(),
                enabled: true,
                enable_range: None,
                effect: Effect::VideoBlur {
                    radius: binding(&scalar),
                },
            },
        },
    });
    project.project.sequences[0].applies.push(apply);

    validate(&project).unwrap();
}

#[test]
fn sequence_owned_clocks_are_valid_only_on_sequence_scoped_sinks() {
    let mut project = sample_project();
    let scalar = bind(&mut project, TemporalType::Scalar, "apply_clock");
    add_sequence_clock(&mut project, "seq_main");
    project.project.sequences[0]
        .applies
        .push(apply_with(binding(&scalar)));
    validate(&project).unwrap();

    let mut other = project.project.sequences[0].clone();
    other.id = SequenceId::new("seq_other").unwrap();
    other.name = "Other".to_owned();
    other.tracks.clear();
    other.applies.clear();
    project.project.sequences.push(other);
    project.temporal.bindings[0].clocks[0].owner = TemporalClockOwner::Sequence {
        sequence_id: SequenceId::new("seq_other").unwrap(),
    };
    assert!(codes(&project).contains(&"TEMPORAL_SINK_CLOCK_OWNER".to_owned()));
}

fn apply_with(opacity: Animatable<f64>) -> Apply {
    Apply {
        id: ApplyId::new("apl_temporal").unwrap(),
        enabled: true,
        record_range: TimeRange::new(time(0), time(300)).unwrap(),
        target: ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
        stages: vec![ApplyStage {
            id: ApplyStageId::new("aps_base").unwrap(),
            enabled: true,
            active_range: None,
            operation: ApplyOperation::Effect {
                effect: EffectInstance {
                    id: EffectId::new("fx_apply_base").unwrap(),
                    enabled: true,
                    enable_range: None,
                    effect: Effect::VideoBlur {
                        radius: Animatable::constant(1.0),
                    },
                },
            },
        }],
        mix: ApplyMix {
            opacity,
            blend_mode: BlendMode::Normal,
            masks: Vec::new(),
        },
    }
}

fn bound_mask(
    scalar: &TemporalBindingId,
    angle: &TemporalBindingId,
    vector: &TemporalBindingId,
) -> Mask {
    let mut mask = Mask {
        shape: MaskShape::Circle,
        position: binding(vector),
        scale: binding(vector),
        rotation_degrees: binding(angle),
        feather_pixels: binding(scalar),
        expansion_pixels: binding(scalar),
        invert: false,
    };
    bind_mask(&mut mask, scalar, angle, vector);
    mask
}

fn bind_mask(
    mask: &mut Mask,
    scalar: &TemporalBindingId,
    angle: &TemporalBindingId,
    vector: &TemporalBindingId,
) {
    mask.position = binding(vector);
    mask.scale = binding(vector);
    mask.rotation_degrees = binding(angle);
    mask.feather_pixels = binding(scalar);
    mask.expansion_pixels = binding(scalar);
}
