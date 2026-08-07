mod support;

use std::collections::BTreeSet;

use support::*;
use veac_plan::canonical::*;
use veac_plan::{required_material_ids_one, resolve_one};

#[test]
fn disabled_effect_apply_matte_does_not_expand_the_plan_closure() {
    let mut envelope = project();
    envelope.project.render_configs[0]
        .raster
        .as_mut()
        .unwrap()
        .captions = CaptionOutput::Discard;
    envelope
        .project
        .materials
        .push(font_material("med_matte_font"));
    envelope.project.sequences[0].tracks.push(track(
        "trk_dormant_matte",
        TrackKind::Caption,
        1,
        vec![caption_matte()],
    ));
    let mut dormant = apply(
        "apl_dormant",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    let ApplyOperation::Effect { effect } = &mut dormant.stages[0].operation else {
        unreachable!()
    };
    effect.enabled = false;
    envelope.project.sequences[0].applies.push(dormant);
    add_apply_matte(
        &mut envelope,
        "rel_dormant_matte",
        "itm_dormant_matte",
        "apl_dormant",
        matte_parameters(TrackMatteMode::Alpha, false),
    );

    let config_id = envelope.project.render_configs[0].id.clone();
    let expected = BTreeSet::from([MaterialId::new("med_video").unwrap()]);
    let required = required_material_ids_one(&envelope, &config_id).unwrap();
    assert_eq!(required, expected);

    let plan = resolve_one(&envelope, &config_id).unwrap();
    let planned: BTreeSet<_> = plan
        .inputs
        .iter()
        .filter_map(|input| input.material_id.clone())
        .collect();
    assert_eq!(planned, expected);
    assert!(plan.sequences[0].applies.is_empty());
    assert!(plan.sequences[0]
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .all(|clip| clip.id.as_str() != "itm_dormant_matte"));
}

fn caption_matte() -> Clip {
    let mut clip = generated_clip("itm_dormant_matte", Generator::Transparent, 0);
    clip.record_range.duration = time(600);
    clip.source = ClipSource::Caption {
        text: "dormant matte".to_owned(),
        speaker: None,
        cue: Box::default(),
        style: text_style(FontRef::Material {
            material_id: MaterialId::new("med_matte_font").unwrap(),
        }),
    };
    clip.visual = Some(visual_properties());
    clip
}
