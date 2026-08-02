use crate::program::compile_source;
use crate::source_edit::{ExpressionSite, SourceNodeRef, SourceSnapshot};

const PROJECT: &str = r#"const time start = 250ms;
const text title = "原始标题";
const scalar amount = 1;
const text asset = "old.png";
project indexed {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  resource image logo { locator local { path ${asset}; } }
  resource font remote-font {
    locator remote {
      uri "https://example.invalid/font.ttf";
      identity sha256 "0000000000000000000000000000000000000000000000000000000000000000";
    }
  }
  sequence main {
    layer visual content {
      item title-card {
        source text { content ${title}; }
        record { at ${start}; duration 2s; }
        state { playback disabled; }
        modifiers {
          effect title-sharpen {
            type video.sharpen;
            parameter amount ${amount};
          }
        }
      }
    }
    layer caption subtitles {
      item caption { source caption { content "字幕"; } record { at 0s; duration 1s; } }
    }
    apply grade {
      scope layer content; record { at 0s; duration 1s; }
      pipeline { stage effect final-grade { type video.color_adjust; parameter contrast 1.1; } }
      mix {}
    }
  }
}"#;

#[test]
fn indexes_authored_project_expression_sites_without_expanding_them() {
    let index = compile_source(PROJECT).unwrap().source_index().unwrap();
    assert_site(
        &index,
        item("content", "title-card"),
        ExpressionSite::ItemRecordStart,
        "${start}",
    );
    assert_site(
        &index,
        item("content", "title-card"),
        ExpressionSite::ItemRecordDuration,
        "2s",
    );
    assert_site(
        &index,
        item("content", "title-card"),
        ExpressionSite::TextContent,
        "${title}",
    );
    assert_site(
        &index,
        SourceNodeRef::resource("main.veac", "logo"),
        ExpressionSite::ResourceLocator,
        "${asset}",
    );
    assert_site(
        &index,
        modifier("content", "title-card", "title-sharpen"),
        ExpressionSite::ModifierParameter {
            parameter: "amount".into(),
        },
        "${amount}",
    );
    assert_site(
        &index,
        item("subtitles", "caption"),
        ExpressionSite::TextContent,
        "\"字幕\"",
    );
    assert_site(
        &index,
        SourceNodeRef::resource("main.veac", "remote-font"),
        ExpressionSite::ResourceLocator,
        "\"https://example.invalid/font.ttf\"",
    );
    assert_site(
        &index,
        SourceNodeRef::stage("main.veac", "indexed", "main", "grade", "final-grade"),
        ExpressionSite::ModifierParameter {
            parameter: "contrast".into(),
        },
        "1.1",
    );
    assert_site(
        &index,
        item("content", "title-card"),
        ExpressionSite::ItemEnabled,
        "disabled",
    );
}

#[test]
fn indexes_typed_container_hierarchy_and_exact_source_ranges() {
    let index = compile_source(PROJECT).unwrap().source_index().unwrap();
    for target in [
        SourceNodeRef::project("main.veac", "indexed"),
        SourceNodeRef::sequence("main.veac", "indexed", "main"),
        SourceNodeRef::layer("main.veac", "indexed", "main", "content"),
        item("content", "title-card"),
        modifier("content", "title-card", "title-sharpen"),
        SourceNodeRef::apply("main.veac", "indexed", "main", "grade"),
        SourceNodeRef::stage("main.veac", "indexed", "main", "grade", "final-grade"),
    ] {
        assert!(index.node_exists(&target), "missing {target:?}");
    }
    let target = modifier("content", "title-card", "title-sharpen");
    let site = ExpressionSite::ModifierParameter {
        parameter: "amount".into(),
    };
    let value = index.expression(&target, &site).unwrap();
    assert_eq!(&PROJECT[value.range.start..value.range.end], value.source);
}

#[test]
fn locally_repeated_modifier_ids_remain_precisely_addressable() {
    let insertion = r#"    layer visual alternate {
      item second-card {
        source text { content "副本"; } record { at 3s; duration 1s; }
        modifiers { effect title-sharpen { type video.sharpen; parameter amount 2; } }
      }
    }
    layer caption subtitles {"#;
    let source = PROJECT.replacen("    layer caption subtitles {", insertion, 1);
    let index = compile_source(&source).unwrap().source_index().unwrap();
    let site = ExpressionSite::ModifierParameter {
        parameter: "amount".into(),
    };
    assert_eq!(
        index
            .expression(&modifier("content", "title-card", "title-sharpen"), &site)
            .unwrap()
            .source,
        "${amount}"
    );
    assert_eq!(
        index
            .expression(
                &modifier("alternate", "second-card", "title-sharpen"),
                &site
            )
            .unwrap()
            .source,
        "2"
    );
}

#[test]
fn curve_modifier_parameters_are_not_pure_expression_sites() {
    let source = PROJECT.replace(
        "parameter amount ${amount};",
        "parameter amount curve { key a { at 0s; value 1; interpolation linear; } }",
    );
    let index = compile_source(&source).unwrap().source_index().unwrap();
    let site = ExpressionSite::ModifierParameter {
        parameter: "amount".into(),
    };
    assert!(index
        .expression(&modifier("content", "title-card", "title-sharpen"), &site)
        .is_none());
}

fn assert_site(
    index: &super::SourceIndex,
    target: SourceNodeRef,
    site: ExpressionSite,
    expected: &str,
) {
    assert_eq!(index.expression(&target, &site).unwrap().source, expected);
}

fn item(layer: &str, item: &str) -> SourceNodeRef {
    SourceNodeRef::item("main.veac", "indexed", "main", layer, item)
}

fn modifier(layer: &str, item: &str, modifier: &str) -> SourceNodeRef {
    SourceNodeRef::modifier("main.veac", "indexed", "main", layer, item, modifier)
}
