use veac_ir::*;

use super::support::{project, time};
use crate::OtioImportResult;

pub(super) fn relation_project() -> ProjectEnvelope {
    let mut envelope = project();
    let track = &mut envelope.project.sequences[0].tracks[0];
    let prototype = track.clips[0].clone();
    for (id, start) in [("itm_next", 840), ("itm_third", 1_380)] {
        let mut clip = prototype.clone();
        clip.id = ItemId::new(id).unwrap();
        clip.record_range = range(start, 600);
        clip.effects.clear();
        track.clips.push(clip);
    }
    envelope.project.relations = vec![
        transition("rel_z_first", "itm_clip", "itm_next"),
        transition("rel_a_second", "itm_next", "itm_third"),
    ];
    validate(&envelope).unwrap();
    envelope
}

pub(super) fn rename_import(imported: &mut OtioImportResult, suffix: &str) {
    let sequence_id = SequenceId::new(format!("seq_{suffix}")).unwrap();
    imported.sequence.id = sequence_id.clone();
    imported.sequence.tracks[0].id = TrackId::new(format!("trk_{suffix}")).unwrap();
    for (index, clip) in imported.sequence.tracks[0].clips.iter_mut().enumerate() {
        clip.id = ItemId::new(format!("itm_{suffix}_{index}")).unwrap();
    }
    for relation in &mut imported.relations {
        relation.sequence_id = sequence_id.clone();
        let RelationKind::Transition { from, to, .. } = &mut relation.kind else {
            continue;
        };
        rename_endpoint(from, suffix);
        rename_endpoint(to, suffix);
    }
}

fn transition(id: &str, from: &str, to: &str) -> Relation {
    Relation {
        id: RelationId::new(id).unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Transition {
            from: RelationEndpoint::item(ItemId::new(from).unwrap()),
            to: RelationEndpoint::item(ItemId::new(to).unwrap()),
            transition: Transition {
                kind: TransitionKind::Dissolve,
                duration: time(60),
                alignment: TransitionAlignment::Centered,
            },
        },
    }
}

fn rename_endpoint(endpoint: &mut RelationEndpoint, suffix: &str) {
    let RelationEndpoint::Item { item_id } = endpoint else {
        return;
    };
    let index = match item_id.as_str() {
        "itm_clip" => 0,
        "itm_next" => 1,
        "itm_third" => 2,
        other => panic!("unexpected endpoint {other}"),
    };
    *item_id = ItemId::new(format!("itm_{suffix}_{index}")).unwrap();
}

fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange {
        start: time(start),
        duration: time(duration),
    }
}
