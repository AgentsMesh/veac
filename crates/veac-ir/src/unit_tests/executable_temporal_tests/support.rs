use crate::*;

pub(super) fn codes(project: &ProjectEnvelope) -> Vec<String> {
    validate(project)
        .unwrap_err()
        .into_diagnostics()
        .into_iter()
        .map(|value| value.code)
        .collect()
}

pub(super) fn bind(
    project: &mut ProjectEnvelope,
    value_type: TemporalType,
    suffix: &str,
) -> TemporalBindingId {
    ensure_provenance(project);
    let value = literal_value(value_type);
    let mut program = TemporalProgram {
        id: TemporalProgramId::new(format!("tpg_{suffix}")).unwrap(),
        opset_version: TEMPORAL_OPSET_VERSION,
        inputs: Vec::new(),
        result_type: value_type,
        nodes: vec![TemporalNode {
            id: TemporalNodeId::new(0),
            value_type,
            kind: TemporalNodeKind::Literal { value },
            provenance_id: None,
        }],
        result: TemporalNodeId::new(0),
        content_sha256: String::new(),
        provenance_id: provenance_id(),
    };
    program.content_sha256 = temporal_program_digest(&program).unwrap();
    let id = TemporalBindingId::new(format!("tbd_{suffix}")).unwrap();
    project.temporal.programs.push(program);
    project.temporal.bindings.push(TemporalBinding {
        id: id.clone(),
        program_id: TemporalProgramId::new(format!("tpg_{suffix}")).unwrap(),
        result_type: value_type,
        clocks: Vec::new(),
        parameters: Vec::new(),
        provenance_id: provenance_id(),
    });
    id
}

pub(super) fn binding<T>(id: &TemporalBindingId) -> Animatable<T> {
    Animatable::Binding {
        binding_id: id.clone(),
    }
}

pub(super) fn add_progress_clock(project: &mut ProjectEnvelope, item_id: &str) {
    let program = &mut project.temporal.programs[0];
    program.inputs.push(TemporalInputDeclaration {
        id: TemporalInputId::new(0),
        value_type: TemporalType::Scalar,
        source: TemporalInputSource::Clock {
            clock: TemporalClock::Progress,
        },
    });
    program.content_sha256 = temporal_program_digest(program).unwrap();
    project.temporal.bindings[0]
        .clocks
        .push(TemporalClockBinding {
            input_id: TemporalInputId::new(0),
            clock: TemporalClock::Progress,
            owner: TemporalClockOwner::Item {
                item_id: ItemId::new(item_id).unwrap(),
            },
        });
}

pub(super) fn add_sequence_clock(project: &mut ProjectEnvelope, sequence_id: &str) {
    let program = &mut project.temporal.programs[0];
    program.inputs.push(TemporalInputDeclaration {
        id: TemporalInputId::new(0),
        value_type: TemporalType::Time,
        source: TemporalInputSource::Clock {
            clock: TemporalClock::SequenceTime,
        },
    });
    program.content_sha256 = temporal_program_digest(program).unwrap();
    project.temporal.bindings[0]
        .clocks
        .push(TemporalClockBinding {
            input_id: TemporalInputId::new(0),
            clock: TemporalClock::SequenceTime,
            owner: TemporalClockOwner::Sequence {
                sequence_id: SequenceId::new(sequence_id).unwrap(),
            },
        });
}

fn ensure_provenance(project: &mut ProjectEnvelope) {
    if project.temporal.provenance.is_empty() {
        project.temporal.provenance.push(provenance());
    }
}

pub(super) fn provenance() -> TemporalProvenance {
    let definition = TemporalDefinitionId::new("def_main").unwrap();
    let source = TemporalSourceId::new("src_main").unwrap();
    TemporalProvenance {
        id: provenance_id(),
        definition: TemporalDefinitionSite {
            id: definition.clone(),
            kind: TemporalDefinitionKind::Function,
            name: "main".to_owned(),
            source_id: source.clone(),
            span: TemporalSourceSpan { start: 0, end: 20 },
        },
        origin: TemporalAuthoredSite {
            definition_id: definition,
            function: "main".to_owned(),
            source_id: source,
            span: TemporalSourceSpan { start: 4, end: 12 },
        },
        call_stack: Vec::new(),
        logical_keys: vec![TemporalLogicalKey::new("key_root").unwrap()],
    }
}

fn provenance_id() -> TemporalProvenanceId {
    TemporalProvenanceId::new("tpv_main").unwrap()
}

fn literal_value(value_type: TemporalType) -> TemporalValue {
    match value_type {
        TemporalType::Scalar => TemporalValue::Scalar { value: 0.5 },
        TemporalType::Angle => TemporalValue::Angle { degrees: 0.0 },
        TemporalType::Vec2 => TemporalValue::Vec2 {
            value: Vec2 { x: 1.0, y: 1.0 },
        },
        TemporalType::Point => TemporalValue::Point {
            value: Point {
                x: Length {
                    value: 0.0,
                    unit: LengthUnit::Pixels,
                },
                y: Length {
                    value: 0.0,
                    unit: LengthUnit::Pixels,
                },
            },
        },
        TemporalType::Rect => TemporalValue::Rect {
            value: Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        },
        _ => panic!("unsupported animation test type"),
    }
}
