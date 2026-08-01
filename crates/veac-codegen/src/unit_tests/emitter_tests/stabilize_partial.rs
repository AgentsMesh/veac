use std::path::PathBuf;

use veac_codegen::emitter::{emit_all, BackendCapabilityKind, BackendFilterBinding};
use veac_plan::canonical::*;
use veac_plan::ResolvedEffect;

use super::support::{bindings, emit_video_command, fixture, resolved, time};

#[test]
fn partial_stabilization_splices_only_the_active_interval() {
    let mut plan = resolved(&fixture());
    plan.sequences[0].tracks[0].clips[0].effects = vec![ResolvedEffect {
        id: EffectId::new("fx_partial_stabilize").unwrap(),
        effect_type: "video.stabilize".into(),
        active_range: TimeRange::new(time(100), time(300)).unwrap(),
        parameters: [("enabled".into(), ParameterValue::Boolean { value: true })]
            .into_iter()
            .collect(),
    }];
    let command = emit_video_command(&plan, &bindings(&plan)).unwrap();
    let graph = command.filter_graph.as_deref().unwrap();
    for marker in [
        "split=3",
        "trim=start=0:end=0.166666666667",
        "trim=start=0.166666666667:end=0.666666666667",
        "vidstabtransform=input=",
        "trim=start=0.666666666667:end=1",
        "concat=n=3:v=1:a=0",
    ] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
    assert_eq!(command.preparations.len(), 1);
    let analysis = command.preparations[0]
        .command
        .filter_graph
        .as_deref()
        .unwrap();
    assert!(analysis.contains("trim=start=0.166666666667:end=0.666666666667"));
    assert!(analysis.contains("vidstabdetect=result="), "{analysis}");
    assert!(!analysis.contains("concat=n=3"), "{analysis}");
    assert_eq!(
        command.preparations[0].outputs,
        [PathBuf::from("stabilize-0000.trf")]
    );
    let filter_bindings = command.preparations[0]
        .command
        .filter_contract
        .as_ref()
        .unwrap()
        .bindings();
    assert!(filter_bindings.iter().any(|value| matches!(
        value,
        BackendFilterBinding::InternalFile { access, .. }
            if *access == veac_codegen::emitter::BackendInternalAccess::Produce
    )));
    let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
    for filter in ["vidstabdetect", "vidstabtransform"] {
        assert!(bundle.requirements().iter().any(|value| {
            value.kind() == BackendCapabilityKind::Filter && value.name() == filter
        }));
    }
}

#[test]
fn sequential_stabilizers_form_an_ordered_internal_artifact_chain() {
    let mut plan = resolved(&fixture());
    plan.sequences[0].tracks[0].clips[0].effects = ["first", "second"]
        .into_iter()
        .map(|name| ResolvedEffect {
            id: EffectId::new(format!("fx_{name}")).unwrap(),
            effect_type: "video.stabilize".into(),
            active_range: TimeRange::new(time(0), time(600)).unwrap(),
            parameters: [("enabled".into(), ParameterValue::Boolean { value: true })]
                .into_iter()
                .collect(),
        })
        .collect();
    let command = emit_video_command(&plan, &bindings(&plan)).unwrap();
    assert_eq!(command.preparations.len(), 2);
    assert_eq!(
        command
            .filter_graph
            .unwrap()
            .matches("vidstabtransform")
            .count(),
        2
    );
    assert_eq!(
        command.preparations[0].outputs,
        [PathBuf::from("stabilize-0000.trf")]
    );
    assert_eq!(
        command.preparations[1].outputs,
        [PathBuf::from("stabilize-0001.trf")]
    );
    let second = command.preparations[1]
        .command
        .filter_contract
        .as_ref()
        .unwrap();
    let accesses = second
        .bindings()
        .iter()
        .filter_map(|binding| match binding {
            BackendFilterBinding::InternalFile { access, .. } => Some(*access),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        accesses,
        [
            veac_codegen::emitter::BackendInternalAccess::Consume,
            veac_codegen::emitter::BackendInternalAccess::Produce,
        ]
    );
}
