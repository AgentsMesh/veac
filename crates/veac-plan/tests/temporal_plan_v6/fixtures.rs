use veac_plan::canonical::*;

use crate::support::{range, visual_properties};

pub fn project_with_clips(enabled: &[bool]) -> (ProjectEnvelope, Vec<TemporalBindingId>) {
    let mut project = crate::support::project();
    install_progress_program(&mut project);
    let template = project.project.sequences[0].tracks[0].clips[0].clone();
    let mut clips = Vec::new();
    let mut ids = Vec::new();
    for (index, enabled) in enabled.iter().copied().enumerate() {
        let item_id = ItemId::new(format!("itm_temporal_{index:03}")).unwrap();
        let binding_id = TemporalBindingId::new(format!("tbd_temporal_{index:03}")).unwrap();
        let mut clip = template.clone();
        clip.id = item_id.clone();
        clip.enabled = enabled;
        clip.record_range = range(index as i64 * 600, 600);
        let mut visual = visual_properties();
        visual.opacity = Animatable::Binding {
            binding_id: binding_id.clone(),
        };
        clip.visual = Some(visual);
        clips.push(clip);
        project.temporal.bindings.push(TemporalBinding {
            id: binding_id.clone(),
            program_id: TemporalProgramId::new("tpg_progress").unwrap(),
            result_type: TemporalType::Scalar,
            clocks: vec![TemporalClockBinding {
                input_id: TemporalInputId::new(0),
                clock: TemporalClock::Progress,
                owner: TemporalClockOwner::Item { item_id },
            }],
            parameters: Vec::new(),
            provenance_id: TemporalProvenanceId::new("tpv_main").unwrap(),
        });
        ids.push(binding_id);
    }
    project.temporal.bindings.reverse();
    project.project.sequences[0].tracks.push(Track {
        id: TrackId::new("trk_temporal").unwrap(),
        kind: TrackKind::Video,
        order: 10,
        placement_mode: PlacementMode::Magnetic,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips,
    });
    (project, ids)
}

pub fn binding<T>(id: &TemporalBindingId) -> Animatable<T> {
    Animatable::Binding {
        binding_id: id.clone(),
    }
}

pub fn provenance(id: &str) -> TemporalProvenance {
    let definition = TemporalDefinitionId::new(format!("def_{id}")).unwrap();
    let source = TemporalSourceId::new(format!("src_{id}")).unwrap();
    TemporalProvenance {
        id: TemporalProvenanceId::new(format!("tpv_{id}")).unwrap(),
        definition: TemporalDefinitionSite {
            id: definition.clone(),
            kind: TemporalDefinitionKind::Function,
            name: id.to_owned(),
            source_id: source.clone(),
            span: TemporalSourceSpan { start: 0, end: 20 },
        },
        origin: TemporalAuthoredSite {
            definition_id: definition,
            function: id.to_owned(),
            source_id: source,
            span: TemporalSourceSpan { start: 4, end: 12 },
        },
        call_stack: Vec::new(),
        logical_keys: vec![TemporalLogicalKey::new(format!("key_{id}")).unwrap()],
    }
}

fn install_progress_program(project: &mut ProjectEnvelope) {
    let provenance = provenance("main");
    let mut program = TemporalProgram {
        id: TemporalProgramId::new("tpg_progress").unwrap(),
        opset_version: TEMPORAL_OPSET_VERSION,
        inputs: vec![TemporalInputDeclaration {
            id: TemporalInputId::new(0),
            value_type: TemporalType::Scalar,
            source: TemporalInputSource::Clock {
                clock: TemporalClock::Progress,
            },
        }],
        result_type: TemporalType::Scalar,
        nodes: vec![TemporalNode {
            id: TemporalNodeId::new(0),
            value_type: TemporalType::Scalar,
            kind: TemporalNodeKind::Input {
                input_id: TemporalInputId::new(0),
            },
            provenance_id: Some(provenance.id.clone()),
        }],
        result: TemporalNodeId::new(0),
        content_sha256: String::new(),
        provenance_id: provenance.id.clone(),
    };
    program.content_sha256 = temporal_program_digest(&program).unwrap();
    project.temporal = TemporalProgramLibrary {
        opset_version: TEMPORAL_OPSET_VERSION,
        programs: vec![program],
        bindings: Vec::new(),
        provenance: vec![provenance],
    };
}

pub fn literal_program(id: &str, value: f64, provenance_id: &str) -> TemporalProgram {
    let mut program = TemporalProgram {
        id: TemporalProgramId::new(id).unwrap(),
        opset_version: TEMPORAL_OPSET_VERSION,
        inputs: Vec::new(),
        result_type: TemporalType::Scalar,
        nodes: vec![TemporalNode {
            id: TemporalNodeId::new(0),
            value_type: TemporalType::Scalar,
            kind: TemporalNodeKind::Literal {
                value: TemporalValue::Scalar { value },
            },
            provenance_id: None,
        }],
        result: TemporalNodeId::new(0),
        content_sha256: String::new(),
        provenance_id: TemporalProvenanceId::new(provenance_id).unwrap(),
    };
    program.content_sha256 = temporal_program_digest(&program).unwrap();
    program
}
