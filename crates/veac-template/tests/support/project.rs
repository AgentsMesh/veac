use veac_ir::*;

use super::video;

pub fn project(fill: FillMode, editable_text: bool) -> ProjectEnvelope {
    let sequence_id = SequenceId::new("seq_main").unwrap();
    let mut tracks = vec![track(
        "trk_video",
        TrackKind::Video,
        0,
        vec![slot_clip(fill)],
    )];
    if editable_text {
        tracks.push(track("trk_text", TrackKind::Visual, 10, vec![text_clip()]));
    }
    let project = envelope(Project {
        id: ProjectId::new("prj_template").unwrap(),
        revision: 4,
        timebase: 600,
        entry_sequence_id: sequence_id.clone(),
        render_configs: vec![RenderConfig {
            id: RenderConfigId::new("out_main").unwrap(),
            sequence_id: sequence_id.clone(),
            raster: Some(RasterSettings {
                width: 1080,
                height: 1920,
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
        }],
        materials: vec![video(time(6000), 1080, 1920)],
        multicam_groups: vec![],
        annotations: vec![],
        relations: vec![],
        sequences: vec![Sequence {
            id: sequence_id,
            name: "Template".to_owned(),
            settings: SequenceSettings {
                width: 1080,
                height: 1920,
                frame_rate: Rational::new(30, 1).unwrap(),
                sample_rate: 48_000,
            },
            tracks,
            applies: vec![],
            authorship: None,
        }],
        applied_operations: vec![],
        authorship: None,
    });
    validate(&project).unwrap();
    project
}

fn envelope(project: Project) -> ProjectEnvelope {
    ProjectEnvelope::new(
        project,
        ExecutableManifest::current(
            env!("CARGO_PKG_VERSION"),
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

pub fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 600).unwrap()
}

fn slot_clip(fill: FillMode) -> Clip {
    Clip {
        id: ItemId::new("itm_slot").unwrap(),
        enabled: true,
        record_range: TimeRange::new(time(0), time(2400)).unwrap(),
        source: ClipSource::Media {
            material_id: MaterialId::new("med_slot").unwrap(),
        },
        source_mapping: Some(SourceMapping::linear(time(0), Rational::new(1, 1).unwrap())),
        visual: Some(visual()),
        audio: None,
        effects: vec![],
        replaceable: Some(SlotConstraint {
            kind: SlotKind::VideoOrImage,
            fill,
            label: "Hero".to_owned(),
            min_source_duration: None,
        }),
        template_editable_text: false,
        authorship: None,
    }
}

fn text_clip() -> Clip {
    Clip {
        id: ItemId::new("itm_title").unwrap(),
        enabled: true,
        record_range: TimeRange::new(time(0), time(1200)).unwrap(),
        source: ClipSource::Text {
            text: "Default title".to_owned(),
            style: TextStyle::default(),
        },
        source_mapping: None,
        visual: Some(visual()),
        audio: None,
        effects: vec![],
        replaceable: Some(SlotConstraint {
            kind: SlotKind::Text,
            fill: FillMode::FitDuration,
            label: "Title".to_owned(),
            min_source_duration: None,
        }),
        template_editable_text: true,
        authorship: None,
    }
}

fn track(id: &str, kind: TrackKind, order: i32, clips: Vec<Clip>) -> Track {
    Track {
        id: TrackId::new(id).unwrap(),
        kind,
        order,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips,
    }
}

fn visual() -> VisualProperties {
    let position = Point {
        x: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
        y: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
    };
    VisualProperties {
        placement: Placement::Absolute { position },
        frame: None,
        transform: Transform2D {
            position: Animatable::constant(position),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            shear: Vec2 { x: 0.0, y: 0.0 },
            flip_horizontal: false,
            flip_vertical: false,
            rotation_degrees: Animatable::constant(0.0),
            anchor: Vec2 { x: 0.5, y: 0.5 },
            crop: None,
        },
        opacity: Animatable::constant(1.0),
        compositing: Compositing {
            z_index: 0,
            blend_mode: BlendMode::Normal,
        },
        masks: vec![],
        card: None,
        color_pipeline: None,
    }
}
