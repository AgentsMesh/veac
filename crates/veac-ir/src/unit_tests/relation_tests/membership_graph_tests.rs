use super::*;

#[test]
fn membership_edges_are_typed_traceable_and_transitive() {
    let project = relation_project();
    let sequence_id = SequenceId::new("seq_main").unwrap();
    let graph = RelationGraph::project(&project.project);

    let group = graph
        .group_for_item(&sequence_id, &item_id("itm_caption"))
        .unwrap();
    assert_eq!(group.relation_id.as_str(), "rel_group");
    assert_eq!(group.sequence_id, &sequence_id);
    assert_eq!(ids(&group.members), ["itm_caption", "itm_video"]);

    let link = graph
        .av_link_for_item(&sequence_id, &item_id("itm_audio"))
        .unwrap();
    assert_eq!(link.relation_id.as_str(), "rel_primary");
    assert_eq!(link.video.clip.id.as_str(), "itm_video");
    assert_eq!(ids(&link.audio), ["itm_audio"]);

    let component = graph
        .membership_component(&sequence_id, &item_id("itm_caption"))
        .unwrap();
    assert_eq!(ids(&component), ["itm_audio", "itm_caption", "itm_video"]);
}

#[test]
fn membership_order_does_not_follow_relation_declaration_order() {
    let mut project = relation_project();
    project.project.relations.reverse();
    let sequence_id = SequenceId::new("seq_main").unwrap();
    let graph = RelationGraph::project(&project.project);
    let group_ids: Vec<_> = graph
        .groups(&sequence_id)
        .iter()
        .map(|edge| edge.relation_id.as_str())
        .collect();
    let link_ids: Vec<_> = graph
        .av_links(&sequence_id)
        .iter()
        .map(|edge| edge.relation_id.as_str())
        .collect();
    assert_eq!(group_ids, ["rel_group"]);
    assert_eq!(link_ids, ["rel_primary"]);
}

#[test]
fn invalid_membership_endpoint_does_not_create_a_partial_edge() {
    let cases = [
        item("itm_missing"),
        RelationEndpoint::track(track_id("trk_audio")),
    ];
    for endpoint in cases {
        let mut project = relation_project();
        let RelationKind::Group { members } = &mut relation_mut(&mut project, "rel_group").kind
        else {
            unreachable!()
        };
        members[1] = endpoint;
        let relation = relation_by_id(&project, "rel_group");
        let graph = RelationGraph::project(&project.project);
        assert!(graph.group(relation).is_none());
        assert!(graph
            .group_for_item(&SequenceId::new("seq_main").unwrap(), &item_id("itm_video"))
            .is_none());
    }
}

#[test]
fn invalid_av_link_endpoint_does_not_create_a_partial_edge() {
    let mut project = relation_project();
    let RelationKind::AvLink { audio, .. } = &mut relation_mut(&mut project, "rel_primary").kind
    else {
        unreachable!()
    };
    audio.push(item("itm_missing"));
    let relation = relation_by_id(&project, "rel_primary");
    let graph = RelationGraph::project(&project.project);
    assert!(graph.av_link(relation).is_none());
    assert!(graph
        .av_link_for_item(&SequenceId::new("seq_main").unwrap(), &item_id("itm_video"))
        .is_none());
}

fn ids<'a>(items: &'a [RelationItem<'_>]) -> Vec<&'a str> {
    let mut ids: Vec<_> = items.iter().map(|item| item.clip.id.as_str()).collect();
    ids.sort_unstable();
    ids
}

fn relation_by_id<'a>(project: &'a ProjectEnvelope, id: &str) -> &'a Relation {
    project
        .project
        .relations
        .iter()
        .find(|relation| relation.id.as_str() == id)
        .unwrap()
}
