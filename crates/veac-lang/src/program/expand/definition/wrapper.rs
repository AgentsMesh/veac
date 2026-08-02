pub(super) const SETTINGS: &str = r#"settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }"#;

pub(super) const TEXT_STYLE: &str = r#"font family "Arial";
            size 16px; fill #ffffffff;"#;

pub(super) const TEXT_LAYOUT: &str = r#"box-width 320px; box-height 80px;
            wrap word; overflow clip;
            horizontal-align center; vertical-align middle;"#;

pub(super) fn project(body: &str) -> String {
    project_with_entry(body, "main")
}

pub(super) fn project_with_entry(body: &str, entry: &str) -> String {
    format!("project validation {{\n  {SETTINGS}\n  entry sequence {entry};\n{body}\n}}")
}

pub(super) fn visual_item(extra: &str) -> String {
    format!(
        r#"  sequence main {{
    layer visual content {{
      item sample {{
        source generated transparent;
        record {{ at 0s; duration 1s; }}
{extra}
      }}
    }}
  }}"#
    )
}
