use super::project;
use crate::authoring::{parse, MappingDecl, SourceDecl};

#[test]
fn parses_every_source_and_mapping_kind() {
    let source = project(
        r#"
  resource video media {
    locator local { path "media.mov"; }
    streams { video auto; audio disabled; }
  }
  resource video alternate {
    locator local { path "alternate.mov"; }
    streams { video auto; audio disabled; }
  }
  multicam interview {
    sync manual { reference angle primary; }
    angle primary { source resource media; source-offset 0s; }
    angle alternate { source resource alternate; source-offset 0s; }
  }
  sequence nested {}
  sequence main {
    layer visual content {
      item media-item {
        source media resource media;
        record { at 0s; duration 3s; }
        mapping linear { from 1s; to 4s; }
      }
      item title {
        source text { content "Hello"; }
        record { at 0s; duration 1s; }
      }
      item solid {
        source generated solid { color #202020ff; }
        record { at 1s; duration 1s; }
        mapping freeze { source 0s; }
      }
      item nested-item {
        source sequence sequence nested;
        record { at 2s; duration 1s; }
        mapping curve {
          key start { at 0s; source 0s; interpolation linear; }
          key end { at 1s; source 2s; interpolation ease-in-out; }
        }
      }
      item multicam-item {
        source multicam multicam interview {
          switch angle primary { at 0s; duration 1s; }
        }
        record { at 3s; duration 1s; }
      }
    }
  }
"#,
    );
    let parsed = parse(&source).unwrap();
    let items = &parsed.project.sequences[1].layers[0].items;
    assert!(matches!(items[0].source, SourceDecl::Media { .. }));
    assert!(matches!(items[1].source, SourceDecl::Text { .. }));
    assert!(items[1].mapping.is_none());
    assert!(matches!(items[2].source, SourceDecl::Generated { .. }));
    assert!(matches!(items[2].mapping, Some(MappingDecl::Freeze { .. })));
    let Some(MappingDecl::Curve { keys, .. }) = &items[3].mapping else {
        panic!("curve mapping expected")
    };
    assert_eq!(keys.len(), 2);
    assert!(matches!(items[4].source, SourceDecl::Multicam { .. }));
}
