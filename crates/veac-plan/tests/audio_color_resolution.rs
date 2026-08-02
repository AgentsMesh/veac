mod support;

use std::collections::BTreeMap;

use support::*;
use veac_plan::canonical::*;
use veac_plan::{plan_hash, resolve_one, ResolvedColorStage, ResolvedInputKind, ResolvedLutKind};

#[test]
fn audio_chain_color_pipeline_and_lut_resource_are_owned_by_the_plan() {
    let mut project = project();
    project.project.materials.push(lut_material());
    project.project.render_configs[0]
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    let mut source_clip = media_clip("itm_sidechain_source", "med_video", 0);
    source_clip.audio = Some(audio_properties());
    project.project.sequences[0].tracks.push(track(
        "trk_sidechain_source",
        TrackKind::Audio,
        10,
        vec![source_clip],
    ));
    let target = &mut project.project.sequences[0].tracks[0].clips[0];
    target.audio = Some(processed_audio());
    target.visual = Some(graded_visual());
    project.project.relations.push(Relation {
        id: RelationId::new("rel_sidechain_resolution").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Sidechain {
            key: RelationEndpoint::track(TrackId::new("trk_sidechain_source").unwrap()),
            target: RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
            parameters: SidechainRelationParameters {
                threshold_db: -24.0,
                ratio: 4.0,
                attack_ms: 10.0,
                release_ms: 250.0,
                active_range: None,
            },
        },
    });

    let output_id = project.project.render_configs[0].id.clone();
    let plan = resolve_one(&project, &output_id).unwrap();
    let target = &plan.sequences[0].tracks[0].clips[0];
    let audio = target.audio.as_ref().unwrap();
    assert_eq!(audio.processors.len(), 2);
    assert!(matches!(
        audio.sidechain.as_ref().unwrap().source,
        SidechainSource::Track { .. }
    ));
    assert_eq!(
        audio.sidechain.as_ref().unwrap().relation_id.as_str(),
        "rel_sidechain_resolution"
    );

    let pipeline = target
        .visual
        .as_ref()
        .unwrap()
        .color_pipeline
        .as_ref()
        .unwrap();
    let ResolvedColorStage::Lut { application } = pipeline.stages.last().unwrap() else {
        panic!("last color stage must be the resolved LUT");
    };
    assert_eq!(application.kind, ResolvedLutKind::ThreeDimensional);
    let resource = plan
        .inputs
        .iter()
        .find(|input| input.id == application.input_id)
        .unwrap();
    assert!(matches!(
        resource.kind,
        ResolvedInputKind::Resource {
            material_kind: MaterialKind::Lut3d
        }
    ));
    assert!(resource.probe.is_none());
    assert_eq!(resource.canonical_uri, "looks/identity.cube");

    let hash = plan_hash(&plan).unwrap();
    assert_eq!(plan_hash(&plan.clone()).unwrap(), hash);
    assert!(!veac_plan::canonical_plan_json(&plan)
        .unwrap()
        .contains("/machine-"));
}

fn processed_audio() -> AudioProperties {
    AudioProperties {
        processors: vec![
            AudioProcessor::ParametricEq {
                bands: vec![ParametricEqBand {
                    frequency_hz: 1_000.0,
                    gain_db: 2.0,
                    q: 1.0,
                }],
            },
            AudioProcessor::Limiter(Limiter {
                ceiling_db: -1.0,
                attack_ms: 5.0,
                release_ms: 50.0,
            }),
        ],
        crossfade: Some(AudioCrossfade {
            fade_in: time(30),
            fade_out: time(30),
            curve: AudioFadeCurve::EqualPower,
        }),
        ..audio_properties()
    }
}

fn graded_visual() -> VisualProperties {
    let mut visual = visual_properties();
    visual.color_pipeline = Some(ColorPipeline {
        input: rec709(),
        working: rec709(),
        output: rec709(),
        stages: vec![
            ColorStage::Basic {
                adjustment: BasicColorAdjustment {
                    exposure_stops: 0.5,
                    temperature_kelvin: 6_500.0,
                    tint: 0.0,
                    highlights: 0.0,
                    shadows: 0.0,
                    fade: 0.0,
                },
            },
            ColorStage::Lut {
                application: LutApplication {
                    material_id: MaterialId::new("med_identity_lut").unwrap(),
                    interpolation: LutInterpolation::Tetrahedral,
                },
            },
        ],
    });
    visual
}

fn lut_material() -> Material {
    Material {
        id: MaterialId::new("med_identity_lut").unwrap(),
        kind: MaterialKind::Lut3d,
        source: MaterialSource::File {
            uri: "looks/identity.cube".to_owned(),
        },
        identity: Some(identity('e')),
        stream_intent: StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Disabled,
        },
        probe: None,
        metadata: BTreeMap::new(),
    }
}

fn rec709() -> ColorSpace {
    ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    }
}
