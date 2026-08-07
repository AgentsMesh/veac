use veac_lang::program::{build_source, DomainOperationId as Op};

const SOURCE: &str = r#"
struct Pair { key: identifier, from: Item, to: Item, }
fn make(pair: Pair, duration: time) -> Relation {
  relation_transition(pair.key, pair.from, pair.to, transition_dissolve(duration))
}
fn main(context: Context) -> Project {
  let labels = map(["一", "二"], fn(value: text) -> text effect pure { value });
  let retained_labels = labels;
  let a = item(identifier("a"), item_enabled(), during(0s, 1500ms),
    source_generated(generator_transparent()), source_timing_native());
  let b = item(identifier("b"), item_enabled(), during(1s, 1s),
    source_generated(generator_transparent()), source_timing_native());
  let c = item(identifier("c"), item_enabled(), during(3s, 1500ms),
    source_generated(generator_transparent()), source_timing_native());
  let d = item(identifier("d"), item_enabled(), during(4s, 1s),
    source_generated(generator_transparent()), source_timing_native());
  let duration = 500ms;
  let first = make(Pair { key: identifier("first"), from: a, to: b, }, duration);
  let second = make(Pair { key: identifier("second"), from: c, to: d, }, duration);
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(a).with_item(b).with_item(c).with_item(d);
  let timeline = sequence(identifier("main"), "关系来源追踪",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(visual).with_relation(first).with_relation(second);
  project(identifier("provenance"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn relation_provenance_survives_helper_and_static_attachment() {
    let built = build_source(SOURCE).unwrap();
    let project = &built.envelope().project;
    assert_eq!(project.relations.len(), 2);
    let sequence = &project.sequences[0];
    let veac_ir::SequenceAuthorship::Veac {
        entity, relations, ..
    } = sequence.authorship.as_ref().unwrap()
    else {
        panic!("expected VEAC authorship")
    };
    let provenance = relations;
    assert_eq!(provenance.len(), 2);
    for relation in &project.relations {
        let value = &provenance
            .iter()
            .find(|value| value.relation_id == relation.id)
            .unwrap()
            .entity;
        let constructor = &value.events[0];
        assert_eq!(constructor.kind, veac_ir::AuthorshipEventKind::Constructor);
        assert_eq!(constructor.operation.0, Op::RelationTransition.opcode());
        assert_eq!(constructor.definition.name.as_str(), "make");
        assert_eq!(
            constructor.definition.kind,
            veac_ir::AuthoredDefinitionKind::Function
        );
        let stack = &constructor.call_stack;
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0].function.as_str(), "main");
    }
    assert!(
        entity
            .events
            .iter()
            .filter(|event| event.kind == veac_ir::AuthorshipEventKind::Update
                && event.operation.0 == Op::SequenceWithRelation.opcode())
            .count()
            == 2
    );
    veac_ir::validate(built.envelope()).unwrap();
}
