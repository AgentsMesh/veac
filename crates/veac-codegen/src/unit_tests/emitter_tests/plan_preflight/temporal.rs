use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;

use super::super::support::{bindings, emit_video_command, fixture, resolved, transition_visual};

#[test]
fn valid_temporal_plan_compiles_the_binding_into_the_ffmpeg_graph() {
    let mut project = fixture();
    install_temporal(&mut project);
    let mut visual = transition_visual();
    visual.opacity = Animatable::Binding {
        binding_id: TemporalBindingId::new("tbd_codegen").unwrap(),
    };
    project.project.sequences[0].tracks[0].clips[0].visual = Some(visual);
    let plan = resolved(&project);

    veac_plan::validate_render_plan(&plan).unwrap();
    let command = emit_video_command(&plan, &bindings(&plan)).unwrap();
    let graph = command.filter_graph.as_deref().unwrap();
    assert!(graph.contains("alpha(X\\,Y)*(0.5)"), "{graph}");
}

#[test]
fn corrupted_plan_contract_is_rechecked_before_backend_traversal() {
    let mut plan = resolved(&fixture());
    plan.header.cache.resolver_sha256 = "0".repeat(64);
    let error = emit_all(&plan, &bindings(&plan)).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == "PLAN_CONTRACT_INVALID"));
}

fn install_temporal(project: &mut ProjectEnvelope) {
    let provenance = provenance();
    let mut program = TemporalProgram {
        id: TemporalProgramId::new("tpg_codegen").unwrap(),
        opset_version: TEMPORAL_OPSET_VERSION,
        inputs: Vec::new(),
        result_type: TemporalType::Scalar,
        nodes: vec![TemporalNode {
            id: TemporalNodeId::new(0),
            value_type: TemporalType::Scalar,
            kind: TemporalNodeKind::Literal {
                value: TemporalValue::Scalar { value: 0.5 },
            },
            provenance_id: Some(provenance.id.clone()),
        }],
        result: TemporalNodeId::new(0),
        content_sha256: String::new(),
        provenance_id: provenance.id.clone(),
    };
    program.content_sha256 = temporal_program_digest(&program).unwrap();
    project.temporal.programs = vec![program];
    project.temporal.bindings = vec![TemporalBinding {
        id: TemporalBindingId::new("tbd_codegen").unwrap(),
        program_id: TemporalProgramId::new("tpg_codegen").unwrap(),
        result_type: TemporalType::Scalar,
        clocks: Vec::new(),
        parameters: Vec::new(),
        provenance_id: provenance.id.clone(),
    }];
    project.temporal.provenance = vec![provenance];
}

fn provenance() -> TemporalProvenance {
    let definition = TemporalDefinitionId::new("def_codegen").unwrap();
    let source = TemporalSourceId::new("src_codegen").unwrap();
    TemporalProvenance {
        id: TemporalProvenanceId::new("tpv_codegen").unwrap(),
        definition: TemporalDefinitionSite {
            id: definition.clone(),
            kind: TemporalDefinitionKind::Function,
            name: "codegen".to_owned(),
            source_id: source.clone(),
            span: TemporalSourceSpan { start: 0, end: 10 },
        },
        origin: TemporalAuthoredSite {
            definition_id: definition,
            function: "codegen".to_owned(),
            source_id: source,
            span: TemporalSourceSpan { start: 1, end: 9 },
        },
        call_stack: Vec::new(),
        logical_keys: vec![TemporalLogicalKey::new("key_codegen").unwrap()],
    }
}
