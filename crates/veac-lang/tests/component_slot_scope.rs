use std::fs;

use tempfile::tempdir;
use veac_lang::program::compile_path;

const COMPONENTS: &str = r##"module {
  const text private_copy = "component-private";
  preset text-style private_style {
    font family "Arial"; size 13px; fill #ffffffff;
  }
  export component sequence shell {
    param text title;
    param time duration default 1s;
    slot text content;
    body {
      layer visual @content {
        item @fill {
          source slot content;
          record { at 0s; duration ${duration}; }
        }
        item @title {
          source text {
            content ${title};
            style { use text-style private_style; }
            layout {
              box-width 320px; box-height 80px; wrap word; overflow clip;
              horizontal-align center; vertical-align middle;
            }
          }
          record { at 0s; duration ${duration}; }
        }
      }
    }
  }
}
"##;

const THEME: &str = r##"module {
  export const text suffix = "-imported";
  export preset text-style heading {
    font family "Arial"; size 31px; fill #22cc88ff;
  }
}
"##;

#[test]
fn fills_use_caller_constants_and_qualified_imported_presets() {
    let source = entry(
        "const text title = \"caller\";",
        r#"source text {
      content ${title + theme.suffix};
      style { use text-style theme.heading; }
      layout {
        box-width 320px; box-height 80px; wrap word; overflow clip;
        horizontal-align center; vertical-align middle;
      }
    }"#,
    );
    let (temp, path) = fixture(&source, COMPONENTS);
    let compiled = compile_path(&path).unwrap();
    let expanded = compiled.expanded_source();
    assert!(expanded.contains("content \"caller-imported\";"));
    assert!(expanded.contains("size 31px"));
    assert!(expanded.contains("content \"bound title\";"));
    assert!(expanded.contains("size 13px"));
    drop(temp);
}

#[test]
fn fills_cannot_capture_component_private_constants_or_presets() {
    for (fill, code) in [
        (
            text_source(
                "${private_copy}",
                "font family \"Arial\"; size 20px; fill #ffffffff;",
            ),
            "PROGRAM_EXPRESSION",
        ),
        (
            text_source("\"copy\"", "use text-style private_style;"),
            "PROGRAM_PRESET_NOT_FOUND",
        ),
    ] {
        let (temp, path) = fixture(&entry("", &fill), COMPONENTS);
        let error = compile_path(&path).unwrap_err();
        assert_eq!(error.as_slice()[0].code, code);
        assert_eq!(error.as_slice()[0].path, "main.veac");
        drop(temp);
    }
}

#[test]
fn component_parameters_are_not_visible_inside_call_site_fills() {
    let source = entry("", &text_source("${title}", valid_style()));
    let expression_start = source.find("${title}").unwrap();
    let (temp, path) = fixture(&source, COMPONENTS);
    let error = compile_path(&path).unwrap_err();
    let diagnostic = &error.as_slice()[0];
    assert_eq!(diagnostic.code, "PROGRAM_EXPRESSION");
    assert_eq!(diagnostic.path, "main.veac");
    assert_eq!(diagnostic.span.start, expression_start);
    assert!(diagnostic.message.contains("unknown symbol `title`"));
    drop(temp);
}

#[test]
fn imported_unused_components_are_validated_in_definition_scope() {
    let broken = COMPONENTS.replace("duration ${duration}", "duration ${missing}");
    let (temp, path) = fixture(&entry_without_instances(), &broken);
    let error = compile_path(&path).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_EXPRESSION");
    assert_eq!(error.as_slice()[0].path, "components.veac");
    drop(temp);
}

fn entry_without_instances() -> String {
    r#"import "./components.veac" as components;
project test {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {}
}"#
    .to_owned()
}

fn entry(declarations: &str, fill: &str) -> String {
    format!(
        r#"import "./components.veac" as components;
import "./theme.veac" as theme;
{declarations}
instance sequence example from components.shell {{
  bind title "bound title";
  fill content {{ {fill} }}
}}
project test {{
  settings {{
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }}
  entry sequence main;
  sequence main {{}}
}}"#
    )
}

fn text_source(content: &str, style: &str) -> String {
    format!(
        r#"source text {{
      content {content}; style {{ {style} }}
      layout {{
        box-width 320px; box-height 80px; wrap word; overflow clip;
        horizontal-align center; vertical-align middle;
      }}
    }}"#
    )
}

fn valid_style() -> &'static str {
    "font family \"Arial\"; size 20px; fill #ffffffff;"
}

fn fixture(entry: &str, components: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let path = temp.path().join("main.veac");
    fs::write(&path, entry).unwrap();
    fs::write(temp.path().join("components.veac"), components).unwrap();
    fs::write(temp.path().join("theme.veac"), THEME).unwrap();
    (temp, path)
}
