#[path = "temporal_bindings/apply.rs"]
mod apply;
#[path = "temporal_bindings/audio.rs"]
mod audio;
#[path = "temporal_bindings/effect.rs"]
mod effect;
#[path = "temporal_bindings/mask.rs"]
mod mask;
#[path = "temporal_bindings/programs.rs"]
mod programs;
#[path = "temporal_bindings/text.rs"]
mod text;
#[path = "temporal_bindings/visual.rs"]
mod visual;

use super::support::*;
use programs::*;

fn install(
    project: &mut ProjectEnvelope,
    name: &str,
    owner: &str,
    result_type: TemporalType,
    nodes: Vec<TemporalNode>,
    result: u32,
) -> TemporalBindingId {
    install_clock(
        project,
        name,
        TemporalType::Scalar,
        TemporalClock::Progress,
        TemporalClockOwner::Item {
            item_id: ItemId::new(owner).unwrap(),
        },
        result_type,
        nodes,
        result,
    )
}

fn install_sequence_time(
    project: &mut ProjectEnvelope,
    name: &str,
    result_type: TemporalType,
    nodes: Vec<TemporalNode>,
    result: u32,
) -> TemporalBindingId {
    install_clock(
        project,
        name,
        TemporalType::Time,
        TemporalClock::SequenceTime,
        TemporalClockOwner::Sequence {
            sequence_id: SequenceId::new("seq_main").unwrap(),
        },
        result_type,
        nodes,
        result,
    )
}

#[allow(clippy::too_many_arguments)]
fn install_clock(
    project: &mut ProjectEnvelope,
    name: &str,
    input_type: TemporalType,
    clock: TemporalClock,
    owner: TemporalClockOwner,
    result_type: TemporalType,
    nodes: Vec<TemporalNode>,
    result: u32,
) -> TemporalBindingId {
    let program_id = TemporalProgramId::new(format!("tpg_{name}")).unwrap();
    let binding_id = TemporalBindingId::new(format!("tbd_{name}")).unwrap();
    let provenance = provenance(name);
    let provenance_id = provenance.id.clone();
    let mut program = TemporalProgram {
        id: program_id.clone(),
        opset_version: TEMPORAL_OPSET_VERSION,
        inputs: vec![TemporalInputDeclaration {
            id: TemporalInputId::new(0),
            value_type: input_type,
            source: TemporalInputSource::Clock { clock },
        }],
        result_type,
        nodes,
        result: TemporalNodeId::new(result),
        content_sha256: String::new(),
        provenance_id: provenance_id.clone(),
    };
    program.content_sha256 = temporal_program_digest(&program).unwrap();
    project.temporal.programs.push(program);
    project.temporal.bindings.push(TemporalBinding {
        id: binding_id.clone(),
        program_id,
        result_type,
        clocks: vec![TemporalClockBinding {
            input_id: TemporalInputId::new(0),
            clock,
            owner,
        }],
        parameters: Vec::new(),
        provenance_id,
    });
    project.temporal.provenance.push(provenance);
    binding_id
}

fn temporal_node(id: u32, value_type: TemporalType, kind: TemporalNodeKind) -> TemporalNode {
    TemporalNode {
        id: TemporalNodeId::new(id),
        value_type,
        kind,
        provenance_id: None,
    }
}

fn provenance(name: &str) -> TemporalProvenance {
    let definition = TemporalDefinitionId::new(format!("def_{name}")).unwrap();
    let source = TemporalSourceId::new(format!("src_{name}")).unwrap();
    TemporalProvenance {
        id: TemporalProvenanceId::new(format!("tpv_{name}")).unwrap(),
        definition: TemporalDefinitionSite {
            id: definition.clone(),
            kind: TemporalDefinitionKind::Function,
            name: name.to_owned(),
            source_id: source.clone(),
            span: TemporalSourceSpan { start: 0, end: 10 },
        },
        origin: TemporalAuthoredSite {
            definition_id: definition,
            function: name.to_owned(),
            source_id: source,
            span: TemporalSourceSpan { start: 1, end: 9 },
        },
        call_stack: Vec::new(),
        logical_keys: vec![TemporalLogicalKey::new(format!("key_{name}")).unwrap()],
    }
}
