use super::*;

#[test]
fn clamps_temporal_bindings_before_directional_filter_commands() {
    let mut project = fixture();
    install_temporal_values(&mut project, &[("angle", 720.0), ("radius", -25.0)]);
    project.project.sequences[0].tracks[0].clips[0].effects = vec![EffectInstance {
        id: EffectId::new("fx_directional_temporal").unwrap(),
        enabled: true,
        enable_range: None,
        effect: Effect::VideoDirectionalBlur {
            angle_degrees: binding("angle"),
            radius: binding("radius"),
        },
    }];
    let plan = resolved(&project);

    let graph = graph(&plan);
    let target = graph
        .split("dblur@")
        .nth(1)
        .and_then(|value| value.split([' ', '=']).next())
        .unwrap();
    assert_runtime_clamp(&graph, target, "angle", 0.0, 360.0);
    assert_runtime_clamp(&graph, target, "radius", 0.0, 100.0);
    assert!(graph.contains("angle clip((720)"), "{graph}");
    assert!(graph.contains("radius clip((-25)"), "{graph}");
}

fn binding(name: &str) -> Animatable<f64> {
    Animatable::Binding {
        binding_id: TemporalBindingId::new(format!("tbd_directional_{name}")).unwrap(),
    }
}

fn install_temporal_values(project: &mut ProjectEnvelope, values: &[(&str, f64)]) {
    let provenance = provenance();
    for (name, value) in values {
        let mut program = TemporalProgram {
            id: TemporalProgramId::new(format!("tpg_directional_{name}")).unwrap(),
            opset_version: TEMPORAL_OPSET_VERSION,
            inputs: Vec::new(),
            result_type: TemporalType::Scalar,
            nodes: vec![TemporalNode {
                id: TemporalNodeId::new(0),
                value_type: TemporalType::Scalar,
                kind: TemporalNodeKind::Literal {
                    value: TemporalValue::Scalar { value: *value },
                },
                provenance_id: Some(provenance.id.clone()),
            }],
            result: TemporalNodeId::new(0),
            content_sha256: String::new(),
            provenance_id: provenance.id.clone(),
        };
        program.content_sha256 = temporal_program_digest(&program).unwrap();
        project.temporal.programs.push(program);
        project.temporal.bindings.push(TemporalBinding {
            id: TemporalBindingId::new(format!("tbd_directional_{name}")).unwrap(),
            program_id: TemporalProgramId::new(format!("tpg_directional_{name}")).unwrap(),
            result_type: TemporalType::Scalar,
            clocks: Vec::new(),
            parameters: Vec::new(),
            provenance_id: provenance.id.clone(),
        });
    }
    project.temporal.provenance = vec![provenance];
}

fn provenance() -> TemporalProvenance {
    let definition = TemporalDefinitionId::new("def_directional_bounds").unwrap();
    let source = TemporalSourceId::new("src_directional_bounds").unwrap();
    TemporalProvenance {
        id: TemporalProvenanceId::new("tpv_directional_bounds").unwrap(),
        definition: TemporalDefinitionSite {
            id: definition.clone(),
            kind: TemporalDefinitionKind::Function,
            name: "directional_bounds".to_owned(),
            source_id: source.clone(),
            span: TemporalSourceSpan { start: 0, end: 20 },
        },
        origin: TemporalAuthoredSite {
            definition_id: definition,
            function: "directional_bounds".to_owned(),
            source_id: source,
            span: TemporalSourceSpan { start: 1, end: 19 },
        },
        call_stack: Vec::new(),
        logical_keys: vec![TemporalLogicalKey::new("key_directional_bounds").unwrap()],
    }
}
