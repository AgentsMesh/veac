use veac_ir::{
    ItemId, MaterialId, TemporalAuthoredSite, TemporalBindingId, TemporalDefinitionId,
    TemporalDefinitionKind, TemporalDefinitionSite, TemporalLogicalKey, TemporalProgramId,
    TemporalProvenance, TemporalProvenanceId, TemporalSourceId, TemporalSourceSpan,
};
use veac_lang::program::expression::{
    compile_temporal_expression, CompiledExpression, CoreTemporalInputIdentity, ExpressionContext,
    ResidualizationLimits, ResidualizationRequest, TypeEnvironment,
};
use veac_lang::program::{
    build_source, prepare_source, ClipTemporalProperty, Diagnostic, ExecutableTemporalLeaf,
    ExecutableTemporalSink,
};

pub const SOLID_SOURCE: &str = r#"
fn main(context: Context) -> Project {
    let first = item(identifier("first"), item_enabled(), during(0s, 3s),
        source_generated(generator_solid(#234567ff)), source_timing_native());
    let second = item(identifier("second"), item_enabled(), during(3s, 3s),
        source_generated(generator_solid(#765432ff)), source_timing_native());
    let state = track_state(track_playback_enabled(), track_audio_audible(),
        track_isolation_normal(), track_editing_unlocked());
    let layer = visual_layer(identifier("visual"), 0, placement_free(), state,
        track_routing_default()).with_item(first).with_item(second);
    let timeline = sequence(identifier("main"), "主时间线",
        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
        .with_layer(layer);
    project(identifier("demo"), project_settings(600))
        .with_sequence(timeline).entry(timeline)
}
"#;

pub const MEDIA_SOURCE: &str = r#"
fn main(context: Context) -> Project {
    let hero = image_resource(identifier("hero"), resource_file("assets/hero.png"),
        sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
    let clip = item(identifier("hero-clip"), item_enabled(), during(0s, 3s),
        source_media(hero), source_timing_native());
    let state = track_state(track_playback_enabled(), track_audio_audible(),
        track_isolation_normal(), track_editing_unlocked());
    let layer = visual_layer(identifier("visual"), 0, placement_free(), state,
        track_routing_default()).with_item(clip);
    let timeline = sequence(identifier("main"), "主时间线",
        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
        .with_layer(layer);
    project(identifier("demo"), project_settings(600)).with_resource(hero)
        .with_sequence(timeline).entry(timeline)
}
"#;

pub struct FixtureIds {
    pub items: Vec<ItemId>,
    pub sequence: veac_ir::SequenceId,
    pub material: Option<MaterialId>,
}

pub fn ids(source: &str) -> FixtureIds {
    let built = build_source(source).unwrap();
    let envelope = built.envelope();
    FixtureIds {
        items: envelope.project.sequences[0].tracks[0]
            .clips
            .iter()
            .map(|clip| clip.id.clone())
            .collect(),
        sequence: envelope.project.sequences[0].id.clone(),
        material: envelope
            .project
            .materials
            .first()
            .map(|value| value.id.clone()),
    }
}

pub fn compile(
    source: &str,
    inputs: impl IntoIterator<Item = (&'static str, CoreTemporalInputIdentity)>,
) -> CompiledExpression {
    let inputs = inputs
        .into_iter()
        .map(|(name, value)| (name.into(), value))
        .collect();
    compile_temporal_expression(
        source,
        &TypeEnvironment::new(),
        &inputs,
        &ExpressionContext::empty(),
    )
    .unwrap()
}

pub fn leaf(
    item: ItemId,
    property: ClipTemporalProperty,
    expression: CompiledExpression,
    name: &str,
) -> ExecutableTemporalLeaf {
    ExecutableTemporalLeaf::new(
        ExecutableTemporalSink::clip(item, property),
        TemporalBindingId::new(format!("tbd_{name}")).unwrap(),
        expression,
        request(name),
    )
}

pub fn execute(source: &str, leaves: &[ExecutableTemporalLeaf]) -> veac_ir::ProjectEnvelope {
    let mut build = prepare_source(source).unwrap();
    for leaf in leaves {
        build.push_temporal_leaf(leaf.clone());
    }
    build.execute().unwrap().envelope().clone()
}

pub fn failure(source: &str, leaves: &[ExecutableTemporalLeaf]) -> Diagnostic {
    let mut build = prepare_source(source).unwrap();
    for leaf in leaves {
        build.push_temporal_leaf(leaf.clone());
    }
    build.execute().unwrap_err().as_slice()[0].clone()
}

pub fn request(name: &str) -> ResidualizationRequest {
    ResidualizationRequest {
        program_id: TemporalProgramId::new(format!("tpg_{name}")).unwrap(),
        provenance: provenance(name),
        limits: ResidualizationLimits::default(),
    }
}

pub fn provenance(name: &str) -> TemporalProvenance {
    let definition_id = TemporalDefinitionId::new(format!("def_{name}")).unwrap();
    let source_id = TemporalSourceId::new(format!("src_{name}")).unwrap();
    let span = TemporalSourceSpan { start: 0, end: 10 };
    TemporalProvenance {
        id: TemporalProvenanceId::new(format!("tpv_{name}")).unwrap(),
        definition: TemporalDefinitionSite {
            id: definition_id.clone(),
            kind: TemporalDefinitionKind::Function,
            name: name.into(),
            source_id: source_id.clone(),
            span,
        },
        origin: TemporalAuthoredSite {
            definition_id,
            function: name.into(),
            source_id,
            span,
        },
        call_stack: Vec::new(),
        logical_keys: vec![TemporalLogicalKey::new(format!("key_{name}")).unwrap()],
    }
}
