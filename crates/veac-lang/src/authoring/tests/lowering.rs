use crate::authoring::{lower_document, parse};
use veac_ir::RelationGraph;

pub(super) const SETTINGS: &str = r#"
  settings {
    timebase 1/1000;
    canvas 1920px by 1080px;
    frame-rate 30/1fps;
    sample-rate 48000hz;
  }
"#;

#[test]
fn lowers_minimal_project_to_valid_ir() {
    let source = format!(
        r#"project demo {{
{SETTINGS}
  entry sequence main;
  sequence main {{
    layer visual canvas {{
      item hero {{
        source generated transparent;
        record {{ at 0s; duration 2s; }}
      }}
    }}
  }}
}}"#
    );
    let envelope = lower_document(&parse(&source).unwrap()).unwrap();
    assert_eq!(envelope.project.sequences.len(), 1);
    assert_eq!(envelope.project.sequences[0].tracks[0].clips.len(), 1);
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn lowers_transition_as_a_relation_fact() {
    let source = format!(
        r#"project demo {{
{SETTINGS}
  entry sequence main;
  sequence main {{
    layer visual canvas {{
      item first {{
        source generated transparent;
        record {{ at 0s; duration 2s; }}
      }}
      item second {{
        source generated transparent;
        record {{ at 2s; duration 2s; }}
      }}
    }}
    relation transition cut {{
      endpoints {{ from item first; to item second; }}
      timing {{ duration 400ms; alignment centered; }}
      style {{ dissolve; }}
    }}
  }}
}}"#
    );
    let envelope = lower_document(&parse(&source).unwrap()).unwrap();
    let sequence = &envelope.project.sequences[0];
    assert_eq!(envelope.project.relations.len(), 1);
    let graph = RelationGraph::project(&envelope.project);
    assert!(graph
        .transition_from(&sequence.id, &sequence.tracks[0].clips[0].id)
        .is_some());
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn lowers_membership_once_as_canonical_relations() {
    let source = format!(
        r#"project demo {{
{SETTINGS}
  entry sequence main;
  sequence main {{
    layer video picture {{
      item camera {{
        source generated transparent;
        record {{ at 0s; duration 2s; }}
      }}
    }}
    layer visual graphics {{
      item badge {{
        source generated transparent;
        record {{ at 0s; duration 2s; }}
      }}
    }}
    layer audio sound {{
      item boom {{
        source generated silence;
        record {{ at 0s; duration 2s; }}
      }}
    }}
    relation group edit-unit {{ members {{ item camera; item badge; }} }}
    relation av-link sync {{
      endpoints {{ video item camera; audio {{ item boom; }} }}
    }}
  }}
}}"#
    );
    let envelope = lower_document(&parse(&source).unwrap()).unwrap();
    let sequence = &envelope.project.sequences[0];
    let graph = RelationGraph::project(&envelope.project);
    assert_eq!(graph.groups(&sequence.id).len(), 1);
    assert_eq!(graph.av_links(&sequence.id).len(), 1);
    let component = graph
        .membership_component(&sequence.id, &sequence.tracks[1].clips[0].id)
        .unwrap();
    assert_eq!(component.len(), 3);
    veac_ir::validate(&envelope).unwrap();
}
