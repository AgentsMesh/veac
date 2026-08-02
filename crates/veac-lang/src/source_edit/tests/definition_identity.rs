use super::*;

#[test]
fn every_source_path_has_a_stable_kind_and_owner_identity() {
    use SourceNodeKind as Kind;
    let module = "defs.veac";
    let cases = [
        case(
            SourceNodeRef::project(module, "project"),
            Kind::Project,
            &["project"],
        ),
        case(
            SourceNodeRef::constant(module, "constant"),
            Kind::Constant,
            &["constant"],
        ),
        case(
            SourceNodeRef::component(module, "card"),
            Kind::Component,
            &["card"],
        ),
        case(
            SourceNodeRef::component_instance(module, "sample"),
            Kind::ComponentInstance,
            &["sample"],
        ),
        case(
            SourceNodeRef::component_local_instance(module, "card", "child"),
            Kind::ComponentLocalInstance,
            &["card", "child"],
        ),
        case(
            SourceNodeRef::component_layer(module, "card", "visual"),
            Kind::ComponentLayer,
            &["card", "visual"],
        ),
        case(
            SourceNodeRef::component_item(module, "card", "visual", "title"),
            Kind::ComponentItem,
            &["card", "visual", "title"],
        ),
        case(
            SourceNodeRef::component_modifier(module, "card", "visual", "title", "blur"),
            Kind::ComponentModifier,
            &["card", "visual", "title", "blur"],
        ),
        case(
            SourceNodeRef::component_apply(module, "card", "finish"),
            Kind::ComponentApply,
            &["card", "finish"],
        ),
        case(
            SourceNodeRef::component_stage(module, "card", "finish", "grade"),
            Kind::ComponentStage,
            &["card", "finish", "grade"],
        ),
        case(
            SourceNodeRef::preset(module, SourcePresetKind::TextStyle, "style"),
            Kind::Preset,
            &["style"],
        ),
        case(
            SourceNodeRef::preset_modifier(module, "stack", "blur"),
            Kind::PresetModifier,
            &["stack", "blur"],
        ),
        case(
            SourceNodeRef::preset_stage(module, "pipeline", "grade"),
            Kind::PresetStage,
            &["pipeline", "grade"],
        ),
        case(
            SourceNodeRef::preset_audio_processor(module, "voice", "final-limiter"),
            Kind::PresetAudioProcessor,
            &["voice", "final-limiter"],
        ),
        case(
            SourceNodeRef::preset_audio_eq_band(module, "voice", "tone-shaper", "presence"),
            Kind::PresetAudioEqBand,
            &["voice", "tone-shaper", "presence"],
        ),
        case(
            SourceNodeRef::preset_delivery_artifact(
                module,
                "delivery",
                SourceDeliveryArtifactKind::AudioStem,
                "voice",
            ),
            Kind::PresetDeliveryArtifact,
            &["delivery", "voice"],
        ),
        case(
            SourceNodeRef::resource(module, "asset"),
            Kind::Resource,
            &["asset"],
        ),
        case(
            SourceNodeRef::sequence(module, "project", "main"),
            Kind::Sequence,
            &["project", "main"],
        ),
        case(
            SourceNodeRef::layer(module, "project", "main", "visual"),
            Kind::Layer,
            &["project", "main", "visual"],
        ),
        case(
            SourceNodeRef::item(module, "project", "main", "visual", "title"),
            Kind::Item,
            &["project", "main", "visual", "title"],
        ),
        case(
            SourceNodeRef::modifier(module, "project", "main", "visual", "title", "blur"),
            Kind::Modifier,
            &["project", "main", "visual", "title", "blur"],
        ),
        case(
            SourceNodeRef::apply(module, "project", "main", "finish"),
            Kind::Apply,
            &["project", "main", "finish"],
        ),
        case(
            SourceNodeRef::stage(module, "project", "main", "finish", "grade"),
            Kind::Stage,
            &["project", "main", "finish", "grade"],
        ),
    ];
    for (target, kind, identifiers) in cases {
        assert_eq!(target.kind(), kind);
        assert_eq!(target.path.identifiers(), identifiers);
    }
}

fn case(
    target: SourceNodeRef,
    kind: SourceNodeKind,
    identifiers: &'static [&'static str],
) -> (SourceNodeRef, SourceNodeKind, &'static [&'static str]) {
    (target, kind, identifiers)
}
