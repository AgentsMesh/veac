use crate::program::diagnostic::Diagnostic;
use crate::program::model::{PresetDecl, PresetKind, SurfaceFile};

use super::{budget::Budget, validate, wrapper};

pub(crate) fn preset(
    file: &SurfaceFile,
    declaration: &PresetDecl,
    body: &str,
    budget: &mut Budget,
) -> Result<(), Diagnostic> {
    let wrapped = match declaration.kind {
        PresetKind::TextStyle => text(body, wrapper::TEXT_LAYOUT),
        PresetKind::TextLayout => text(wrapper::TEXT_STYLE, body),
        PresetKind::ModifierStack => wrapper::project(&wrapper::visual_item(&format!(
            "        modifiers {{\n{body}\n        }}"
        ))),
        PresetKind::EffectPipeline => apply(body),
        PresetKind::ColorPipeline => apply(&format!("stage color validation {{\n{body}\n}}")),
        PresetKind::AudioProcessors => audio(body),
        PresetKind::DeliveryProfile => delivery(body),
    };
    validate::source(&wrapped).map_err(|message| {
        Diagnostic::new(
            "PROGRAM_PRESET_DEFINITION",
            &file.path,
            format!(
                "invalid {} preset `{}`: {message}",
                kind(declaration.kind),
                declaration.name
            ),
            declaration.span,
        )
    })?;
    budget.preset(&file.path, declaration.span, body.len(), wrapped.len())
}

fn text(style: &str, layout: &str) -> String {
    wrapper::project(&format!(
        r#"  sequence main {{ layer visual content {{ item sample {{
    source text {{ content "validation"; style {{ {style} }} layout {{ {layout} }} }}
    record {{ at 0s; duration 1s; }}
  }} }} }}"#
    ))
}

fn apply(body: &str) -> String {
    wrapper::project(&format!(
        r#"  sequence main {{
    layer visual content {{
      item sample {{ source generated transparent; record {{ at 0s; duration 1s; }} }}
    }}
    apply validation {{
      scope layer content; record {{ at 0s; duration 1s; }}
      pipeline {{ {body} }} mix {{}}
    }}
  }}"#
    ))
}

fn audio(body: &str) -> String {
    wrapper::project(&format!(
        r#"  sequence main {{ layer audio content {{ item sample {{
    source generated silence; record {{ at 0s; duration 1s; }}
    modifiers {{ audio validation {{ {body} }} }}
  }} }} }}"#
    ))
}

fn delivery(body: &str) -> String {
    wrapper::project(&format!(
        "  sequence main {{}}\n  delivery validation {{ sequence main;\n{body}\n  }}"
    ))
}

fn kind(value: PresetKind) -> &'static str {
    crate::program::parser::kind::preset_name(value)
}
