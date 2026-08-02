use super::*;

#[test]
fn typed_audio_chain_crossfade_sidechain_and_color_pipeline_are_editable() {
    let project = linked_project();
    let clip_id = ItemId::new("itm_video").unwrap();
    let processors = vec![AudioProcessor {
        id: AudioProcessorId::new("aud_final-limiter").unwrap(),
        kind: AudioProcessorKind::Limiter(Limiter {
            ceiling_db: -1.0,
            attack_ms: 5.0,
            release_ms: 50.0,
        }),
    }];
    let crossfade = AudioCrossfade {
        fade_in: time(30),
        fade_out: time(30),
        curve: AudioFadeCurve::EqualPower,
    };
    let sidechain = Relation {
        id: RelationId::new("rel_sidechain_edit").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Sidechain {
            key: RelationEndpoint::track(TrackId::new("trk_audio").unwrap()),
            target: RelationEndpoint::item(clip_id.clone()),
            parameters: SidechainRelationParameters {
                threshold_db: -24.0,
                ratio: 4.0,
                attack_ms: 10.0,
                release_ms: 250.0,
                active_range: None,
            },
        },
    };
    let color_pipeline = ColorPipeline {
        input: rec709(),
        working: rec709(),
        output: rec709(),
        stages: vec![ColorStage::Basic {
            adjustment: BasicColorAdjustment {
                exposure_stops: 0.5,
                temperature_kelvin: 6_500.0,
                tint: 0.0,
                highlights: 0.0,
                shadows: 0.0,
                fade: 0.0,
            },
        }],
    };
    let operations = vec![
        EditOperation::SetAudioProperty {
            clip_id: clip_id.clone(),
            property: AudioProperty::Processors(processors.clone()),
        },
        EditOperation::SetAudioProperty {
            clip_id: clip_id.clone(),
            property: AudioProperty::Crossfade(Some(crossfade)),
        },
        EditOperation::EditStructure {
            edit: StructureEdit::InsertRelation {
                relation: sidechain.clone(),
            },
        },
        EditOperation::SetVisualProperty {
            clip_id,
            property: VisualProperty::ColorPipeline(Some(color_pipeline.clone())),
        },
    ];
    let edit = batch("op_audio_color_properties", &project, operations);
    let (updated, changed_objects) = match apply_edit_batch(&project, &edit) {
        EditOutcome::Applied {
            project,
            changed_objects,
            ..
        } => (project, changed_objects),
        other => panic!("expected applied edit, got {other:?}"),
    };
    let clip = &updated.project.sequences[0].tracks[0].clips[0];
    let audio = clip.audio.as_ref().unwrap();
    assert_eq!(audio.processors, processors);
    assert_eq!(audio.crossfade, Some(crossfade));
    assert!(updated.project.relations.contains(&sidechain));
    assert_eq!(
        clip.visual.as_ref().unwrap().color_pipeline,
        Some(color_pipeline)
    );
    assert!(changed_objects.contains(&ChangedObjectId::Item {
        id: ItemId::new("itm_video").unwrap(),
    }));
}

fn rec709() -> ColorSpace {
    ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    }
}
