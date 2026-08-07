use crate::*;

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

pub(super) fn provenance_id() -> TemporalProvenanceId {
    TemporalProvenanceId::new("tpv_main").unwrap()
}

pub(super) fn scalar(value: f64) -> TemporalValue {
    TemporalValue::Scalar { value }
}

pub(super) fn literal(id: u32, value: TemporalValue) -> TemporalNode {
    TemporalNode {
        id: TemporalNodeId::new(id),
        value_type: value.value_type(),
        kind: TemporalNodeKind::Literal { value },
        provenance_id: None,
    }
}

pub(super) fn program(
    inputs: Vec<TemporalInputDeclaration>,
    nodes: Vec<TemporalNode>,
    result: u32,
    result_type: TemporalType,
) -> TemporalProgram {
    let mut program = TemporalProgram {
        id: TemporalProgramId::new("tpg_main").unwrap(),
        opset_version: TEMPORAL_OPSET_VERSION,
        inputs,
        result_type,
        nodes,
        result: TemporalNodeId::new(result),
        content_sha256: String::new(),
        provenance_id: provenance_id(),
    };
    seal(&mut program);
    program
}

pub(super) fn seal(program: &mut TemporalProgram) {
    program.content_sha256 = temporal_program_digest(program).unwrap();
}

pub(super) fn progress_program() -> TemporalProgram {
    let inputs = vec![TemporalInputDeclaration {
        id: TemporalInputId::new(0),
        value_type: TemporalType::Scalar,
        source: TemporalInputSource::Clock {
            clock: TemporalClock::Progress,
        },
    }];
    let nodes = vec![
        TemporalNode {
            id: TemporalNodeId::new(0),
            value_type: TemporalType::Scalar,
            kind: TemporalNodeKind::Input {
                input_id: TemporalInputId::new(0),
            },
            provenance_id: None,
        },
        literal(1, scalar(2.0)),
        TemporalNode {
            id: TemporalNodeId::new(2),
            value_type: TemporalType::Scalar,
            kind: TemporalNodeKind::Binary {
                operation: TemporalBinaryOperation::Multiply,
                left: TemporalNodeId::new(0),
                right: TemporalNodeId::new(1),
            },
            provenance_id: None,
        },
    ];
    program(inputs, nodes, 2, TemporalType::Scalar)
}

pub(super) fn progress_binding() -> TemporalBinding {
    TemporalBinding {
        id: TemporalBindingId::new("tbd_main").unwrap(),
        program_id: TemporalProgramId::new("tpg_main").unwrap(),
        result_type: TemporalType::Scalar,
        clocks: vec![TemporalClockBinding {
            input_id: TemporalInputId::new(0),
            clock: TemporalClock::Progress,
            owner: TemporalClockOwner::Item {
                item_id: ItemId::new("itm_subject").unwrap(),
            },
        }],
        parameters: Vec::new(),
        provenance_id: provenance_id(),
    }
}

pub(super) fn library() -> TemporalProgramLibrary {
    TemporalProgramLibrary {
        opset_version: TEMPORAL_OPSET_VERSION,
        programs: vec![progress_program()],
        bindings: vec![progress_binding()],
        provenance: vec![provenance()],
    }
}

pub(super) fn codes(error: TemporalValidationErrors) -> Vec<String> {
    error
        .into_diagnostics()
        .into_iter()
        .map(|value| value.code)
        .collect()
}
