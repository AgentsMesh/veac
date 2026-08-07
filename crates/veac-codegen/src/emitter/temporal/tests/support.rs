use std::process::Command;

use veac_plan::canonical::*;
use veac_plan::ResolvedRenderPlan;

use super::super::{compile_binding, evaluate_binding, CompiledValue};
use crate::emitter::process_owner::ProcessOwner;
use crate::unit_tests::emitter_tests::support::{fixture, resolved};

pub(in crate::emitter) fn plan(
    mut program: TemporalProgram,
    clocks: Vec<(TemporalInputId, TemporalClock)>,
    parameters: Vec<(TemporalInputId, TemporalParameterId, TemporalValue)>,
) -> (ResolvedRenderPlan, TemporalBindingId) {
    program.content_sha256 = temporal_program_digest(&program).unwrap();
    let mut plan = resolved(&fixture());
    let item_id = plan.sequences[0].tracks[0].clips[0].id.clone();
    let sequence_id = plan.sequences[0].id.clone();
    let binding_id = TemporalBindingId::new("tbd_backend_test").unwrap();
    let binding = TemporalBinding {
        id: binding_id.clone(),
        program_id: program.id.clone(),
        result_type: program.result_type,
        clocks: clocks
            .into_iter()
            .map(|(input_id, clock)| TemporalClockBinding {
                input_id,
                clock,
                owner: match clock {
                    TemporalClock::SequenceTime | TemporalClock::Frame => {
                        TemporalClockOwner::Sequence {
                            sequence_id: sequence_id.clone(),
                        }
                    }
                    _ => TemporalClockOwner::Item {
                        item_id: item_id.clone(),
                    },
                },
            })
            .collect(),
        parameters: parameters
            .into_iter()
            .map(|(input_id, parameter_id, value)| TemporalParameterBinding {
                input_id,
                parameter_id,
                value,
            })
            .collect(),
        provenance_id: provenance_id(),
    };
    plan.temporal = TemporalProgramLibrary {
        opset_version: TEMPORAL_OPSET_VERSION,
        programs: vec![program],
        bindings: vec![binding],
        provenance: Vec::new(),
    };
    (plan, binding_id)
}

pub(in crate::emitter) fn program(
    inputs: Vec<TemporalInputDeclaration>,
    nodes: Vec<TemporalNode>,
    result: u32,
    result_type: TemporalType,
) -> TemporalProgram {
    TemporalProgram {
        id: TemporalProgramId::new("tpg_backend_test").unwrap(),
        opset_version: TEMPORAL_OPSET_VERSION,
        inputs,
        result_type,
        nodes,
        result: TemporalNodeId::new(result),
        content_sha256: String::new(),
        provenance_id: provenance_id(),
    }
}

pub(in crate::emitter) fn node(
    id: u32,
    value_type: TemporalType,
    kind: TemporalNodeKind,
) -> TemporalNode {
    TemporalNode {
        id: TemporalNodeId::new(id),
        value_type,
        kind,
        provenance_id: None,
    }
}

pub(super) fn compile(
    plan: &ResolvedRenderPlan,
    binding_id: &TemporalBindingId,
    clock: &str,
) -> Result<CompiledValue, super::super::TemporalBackendError> {
    let clip = &plan.sequences[0].tracks[0].clips[0];
    compile_binding(plan, binding_id, ProcessOwner::clip(clip), clock)
}

pub(super) fn evaluate(
    plan: &ResolvedRenderPlan,
    binding_id: &TemporalBindingId,
    seconds: f64,
) -> TemporalValue {
    let clip = &plan.sequences[0].tracks[0].clips[0];
    evaluate_binding(plan, binding_id, ProcessOwner::clip(clip), seconds).unwrap()
}

pub(super) fn ffmpeg(expression: &str) -> f64 {
    let source = format!("aevalsrc=exprs='{expression}':s=8000:d=0.001");
    let output = Command::new("ffmpeg")
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            &source,
            "-frames:a",
            "1",
            "-c:a",
            "pcm_f64le",
            "-f",
            "f64le",
            "pipe:1",
        ])
        .output()
        .expect("focused FFmpeg differential test requires ffmpeg");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    f64::from_le_bytes(output.stdout[..8].try_into().unwrap())
}

pub(super) fn numeric_expression(value: &CompiledValue) -> &str {
    match value {
        CompiledValue::Boolean(value)
        | CompiledValue::Integer(value)
        | CompiledValue::Scalar(value)
        | CompiledValue::Time(value)
        | CompiledValue::Angle(value) => value,
        CompiledValue::Length(value) => &value.value,
        _ => panic!("expected a numeric compiled value"),
    }
}

pub(super) fn numeric_value(value: &TemporalValue) -> f64 {
    match value {
        TemporalValue::Boolean { value } => f64::from(u8::from(*value)),
        TemporalValue::Integer { value } => *value as f64,
        TemporalValue::Scalar { value } => *value,
        TemporalValue::Time { value } => value.value as f64 / f64::from(value.timescale),
        TemporalValue::Length { value } => value.value,
        TemporalValue::Angle { degrees } => *degrees,
        _ => panic!("expected a numeric temporal value"),
    }
}

pub(super) fn assert_code(
    result: Result<CompiledValue, super::super::TemporalBackendError>,
    expected: &'static str,
) {
    assert_eq!(result.unwrap_err().code, expected);
}

pub(super) fn scalar(value: f64) -> TemporalValue {
    TemporalValue::Scalar { value }
}

fn provenance_id() -> TemporalProvenanceId {
    TemporalProvenanceId::new("tpv_backend_test").unwrap()
}
