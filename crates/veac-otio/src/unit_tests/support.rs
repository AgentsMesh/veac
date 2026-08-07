use veac_ir::*;

pub(super) fn project() -> ProjectEnvelope {
    let sequence_id = SequenceId::new("seq_main").unwrap();
    let material = material("med_video", "media/video.mp4", MaterialKind::Video);
    envelope(Project {
        id: ProjectId::new("prj_otio").unwrap(),
        revision: 3,
        timebase: 600,
        entry_sequence_id: sequence_id.clone(),
        render_configs: vec![render_config(sequence_id.clone())],
        materials: vec![material],
        multicam_groups: vec![],
        annotations: vec![],
        relations: vec![],
        sequences: vec![Sequence {
            id: sequence_id,
            name: "Main".to_owned(),
            settings: settings(),
            tracks: vec![Track {
                id: TrackId::new("trk_video").unwrap(),
                kind: TrackKind::Video,
                order: 0,
                placement_mode: PlacementMode::Free,
                state: state(),
                routing: TrackRouting::Default,
                clips: vec![clip()],
            }],
            applies: vec![],
            authorship: None,
        }],
        applied_operations: vec![],
        authorship: None,
    })
}

fn envelope(project: Project) -> ProjectEnvelope {
    ProjectEnvelope::new(
        project,
        ExecutableManifest::current(
            "0.1.0",
            ExecutableDigests {
                domain_registry_sha256: "a".repeat(64),
                main_core_sha256: "b".repeat(64),
                source_graph_sha256: "c".repeat(64),
                declared_inputs_sha256: "d".repeat(64),
                compiler_sha256: "e".repeat(64),
            },
        ),
        TemporalProgramLibrary {
            opset_version: TEMPORAL_OPSET_VERSION,
            programs: vec![],
            bindings: vec![],
            provenance: vec![],
        },
    )
}

pub(super) fn settings() -> SequenceSettings {
    SequenceSettings {
        width: 1920,
        height: 1080,
        frame_rate: Rational::new(30, 1).unwrap(),
        sample_rate: 48_000,
    }
}

pub(super) fn material(id: &str, uri: &str, kind: MaterialKind) -> Material {
    Material {
        id: MaterialId::new(id).unwrap(),
        kind,
        source: MaterialSource::File {
            uri: uri.to_owned(),
        },
        identity: None,
        stream_intent: StreamIntent {
            video: StreamChoice::Auto,
            audio: StreamChoice::Disabled,
        },
        probe: None,
        authorship: None,
    }
}

pub(super) fn clip() -> Clip {
    Clip {
        id: ItemId::new("itm_clip").unwrap(),
        enabled: true,
        record_range: TimeRange::new(time(300), time(600)).unwrap(),
        source: ClipSource::Media {
            material_id: MaterialId::new("med_video").unwrap(),
        },
        source_mapping: Some(SourceMapping::linear(
            time(1200),
            Rational::new(1, 1).unwrap(),
        )),
        visual: Some(crate::import::defaults::visual(0)),
        audio: None,
        effects: vec![],
        replaceable: None,
        template_editable_text: false,
        authorship: None,
    }
}

fn render_config(sequence_id: SequenceId) -> RenderConfig {
    RenderConfig {
        id: RenderConfigId::new("out_main").unwrap(),
        sequence_id,
        raster: Some(RasterSettings {
            width: 1920,
            height: 1080,
            frame_rate: Rational::new(30, 1).unwrap(),
            captions: CaptionOutput::BurnIn,
        }),
        deliverables: vec![Deliverable {
            id: DeliverableId::new("dlv_main").unwrap(),
            target: DeliverableTarget::File {
                name: "main.mp4".to_owned(),
            },
            kind: DeliverableKind::Video(VideoDeliverable::default()),
        }],
    }
}

fn state() -> TrackState {
    TrackState {
        enabled: true,
        muted: false,
        solo: false,
        locked: false,
    }
}

pub(super) fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 600).unwrap()
}
