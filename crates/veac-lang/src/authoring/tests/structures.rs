use super::project;
use crate::authoring::{
    parse, ApplyItemTarget, ApplyScope, ApplyStageDecl, ModifierDecl, ParameterDecl, StructureDecl,
};

#[test]
fn preserves_typed_modifier_and_structure_blocks() {
    let source = project(
        r#"
  resource video media {
    locator local { path "media.mov"; }
    streams { video auto; audio disabled; }
  }
  sequence main {
    layer video primary {
      item hero {
        source media resource media;
        record { at 0s; duration 2s; }
        modifiers {
          transform hero-transform {
            position { x 0.5; y -0.5; }
            scale { x 1; y 1; }
            rotation +0.5deg;
            flip-horizontal true;
          }
          composite hero-composite {
            opacity 0.8;
            z-index 2;
            blend screen;
          }
        }
      }
      item voice {
        source generated silence;
        record { at 0s; duration 2s; }
      }
    }
  relation transition intro-cut {
    endpoints { from item hero; to item voice; }
    timing { duration 300ms; alignment centered; }
    style { dissolve; }
  }
  apply cinematic {
    scope items { item hero; }
    record { at 0s; duration 2s; }
    pipeline {
      stage effect grade {
        type video.color_adjust;
        parameter contrast 1.2;
      }
    }
    mix { opacity 0.8; blend screen; }
  }
  }
  annotation marker shot-start {
    target sequence main;
    span range { at 0s; duration 2s; }
    payload { label "Shot start"; color #ffcc00ff; }
    provenance {
      producer "test";
      request-sha256 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
      response-sha256 "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    }
  }
  delivery master {
    sequence main;
    raster { canvas 1920px by 1080px; frame-rate 30fps; captions burn-in; }
    artifact video master {
      target file "master.mp4";
      mux mp4 { video h264 {} audio none; }
    }
  }
"#,
    );
    let parsed = parse(&source).unwrap();
    let modifier = &parsed.project.sequences[0].layers[0].items[0].modifiers[0];
    let ModifierDecl::Transform(modifier) = modifier else {
        panic!("transform expected")
    };
    assert_eq!(modifier.id.value, "hero-transform");
    let Some(ParameterDecl::Constant(rotation)) = &modifier.rotation else {
        panic!("constant rotation expected")
    };
    assert_eq!(rotation.raw, "+0.5deg");
    let structures = &parsed.project.sequences[0].structures;
    assert_eq!(structures.len(), 2);
    assert_eq!(parsed.project.annotations.len(), 1);
    assert_eq!(parsed.project.deliveries.len(), 1);
    let StructureDecl::Relation(relation) = &structures[0] else {
        panic!("relation expected")
    };
    let crate::authoring::RelationKind::Transition(transition) = &relation.kind else {
        panic!("transition expected")
    };
    assert_eq!(transition.endpoints.from.id.value, "hero");
    assert_eq!(transition.endpoints.to.id.value, "voice");
    assert_eq!(transition.timing.duration.raw, "300ms");
    let StructureDecl::Apply(apply) = &structures[1] else {
        panic!("apply expected")
    };
    let ApplyScope::Items { targets, .. } = &apply.scope else {
        panic!("items scope expected")
    };
    assert!(matches!(&targets[0], ApplyItemTarget::Item(id) if id.value == "hero"));
    assert!(
        matches!(&apply.pipeline[0], ApplyStageDecl::Effect(value) if value.id.value == "grade")
    );
    assert_eq!(apply.mix.blend.as_ref().unwrap().value, "screen");
    assert_eq!(parsed.project.deliveries[0].sequence.value, "main");
}
