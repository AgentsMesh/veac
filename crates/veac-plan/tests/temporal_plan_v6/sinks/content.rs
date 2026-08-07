use veac_plan::canonical::*;

use crate::support;

use super::library::Bindings;

pub(super) fn bind_media_clip(clip: &mut Clip, ids: &Bindings) {
    let mut visual = support::visual_properties();
    visual.transform.position = binding(&ids.point);
    visual.transform.scale = binding(&ids.vector);
    visual.transform.rotation_degrees = binding(&ids.angle);
    visual.transform.crop = Some(binding(&ids.rect));
    visual.opacity = binding(&ids.scalar);
    visual.masks.push(mask(ids));
    clip.visual = Some(visual);
    let mut audio = support::audio_properties();
    audio.gain = binding(&ids.scalar);
    audio.pan = binding(&ids.scalar);
    clip.audio = Some(audio);
    clip.effects = vec![effect("fx_clip", binding(&ids.scalar))];
}

pub(super) fn add_text_clip(project: &mut ProjectEnvelope, mut clip: Clip, ids: &Bindings) {
    let font_id = MaterialId::new("med_temporal_font").unwrap();
    project
        .project
        .materials
        .push(support::font_material(font_id.as_str()));
    let mut style = support::text_style(FontRef::Material {
        material_id: font_id,
    });
    style.animation = Some(text_animation(ids));
    clip.id = ItemId::new("itm_temporal_text").unwrap();
    clip.source = ClipSource::Text {
        text: "Temporal sinks".to_owned(),
        style,
    };
    clip.source_mapping = None;
    clip.visual = Some(support::visual_properties());
    clip.audio = None;
    clip.effects.clear();
    project.project.sequences[0].tracks.push(Track {
        id: TrackId::new("trk_temporal_text").unwrap(),
        kind: TrackKind::Visual,
        order: 1,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips: vec![clip],
    });
}

pub(super) fn add_apply(project: &mut ProjectEnvelope, ids: &Bindings) {
    let mut apply = support::apply(
        "apl_temporal",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    apply.mix.opacity = binding(&ids.scalar);
    apply.mix.masks.push(mask(ids));
    let ApplyOperation::Effect { effect } = &mut apply.stages[0].operation else {
        unreachable!()
    };
    effect.effect = Effect::VideoBlur {
        radius: binding(&ids.scalar),
    };
    project.project.sequences[0].applies.push(apply);
}

fn text_animation(ids: &Bindings) -> TextAnimation {
    TextAnimation {
        granularity: TextGranularity::Word,
        transform: TextUnitTransform {
            position_offset: binding(&ids.point),
            scale: binding(&ids.vector),
            rotation_degrees: binding(&ids.angle),
        },
        reveal: binding(&ids.scalar),
        highlight: Some(TextHighlightAnimation {
            fill: Color {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 255,
            },
            progress: binding(&ids.scalar),
        }),
        opacity: binding(&ids.scalar),
        stagger: support::time(0),
    }
}

fn mask(ids: &Bindings) -> Mask {
    Mask {
        shape: MaskShape::Circle,
        position: binding(&ids.vector),
        scale: binding(&ids.vector),
        rotation_degrees: binding(&ids.angle),
        feather_pixels: binding(&ids.scalar),
        expansion_pixels: binding(&ids.scalar),
        invert: false,
    }
}

fn effect(id: &str, value: Animatable<f64>) -> EffectInstance {
    EffectInstance {
        id: EffectId::new(id).unwrap(),
        enabled: true,
        enable_range: None,
        effect: Effect::VideoBlur { radius: value },
    }
}

fn binding<T>(id: &TemporalBindingId) -> Animatable<T> {
    Animatable::Binding {
        binding_id: id.clone(),
    }
}
