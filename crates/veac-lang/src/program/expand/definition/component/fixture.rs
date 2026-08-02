use crate::program::expression::ValueType;
use crate::program::model::ResolvedComponent;

use super::super::wrapper;

pub(super) const IDENTIFIER_BINDING: &str = "__veac_identifier";

#[derive(Clone, Copy)]
pub(super) struct Environment {
    pub instance: &'static str,
    pub entry: &'static str,
    pub video: &'static str,
    pub audio: &'static str,
    pub sequence: &'static str,
    pub identifier: &'static str,
}

pub(super) const ENVIRONMENTS: [Environment; 2] = [
    Environment {
        instance: "validation-first-instance",
        entry: "validation-first-entry",
        video: "validation-first-video",
        audio: "validation-first-audio",
        sequence: "validation-first-sequence",
        identifier: "validation-first-identifier",
    },
    Environment {
        instance: "validation-second-instance",
        entry: "validation-second-entry",
        video: "validation-second-video",
        audio: "validation-second-audio",
        sequence: "validation-second-sequence",
        identifier: "validation-second-identifier",
    },
];

pub(super) fn source(component: &ResolvedComponent, names: Environment) -> String {
    let mut bindings = String::new();
    for parameter in &component.declaration.parameters {
        if parameter.default.is_none() {
            bindings.push_str(&format!(
                "  bind {} {};\n",
                parameter.name,
                sentinel(parameter.value_type)
            ));
        }
    }
    for slot in &component.declaration.slots {
        bindings.push_str(&format!(
            "  fill {} {{ {} }}\n",
            slot.name,
            slot_source(slot.kind, names)
        ));
    }
    let body = validation_body(names);
    format!(
        "instance sequence {} from {} {{\n{bindings}}}\n{}",
        names.instance,
        component.declaration.name,
        wrapper::project_with_entry(&body, names.entry)
    )
}

fn validation_body(names: Environment) -> String {
    format!(
        r#"  resource video {} {{
    locator local {{ path "validation.mp4"; }}
    streams {{ video auto; audio disabled; }}
  }}
  resource audio {} {{
    locator local {{ path "validation.wav"; }}
    streams {{ video disabled; audio auto; }}
  }}
  resource video {} {{
    locator local {{ path "validation.mov"; }}
    streams {{ video auto; audio auto; }}
  }}
  sequence {} {{}}
  sequence {} {{}}
  sequence {} {{}}"#,
        names.video, names.audio, names.identifier, names.entry, names.sequence, names.identifier
    )
}

fn sentinel(kind: ValueType) -> &'static str {
    match kind {
        ValueType::Scalar => "1",
        ValueType::Time => "1s",
        ValueType::Length => "1px",
        ValueType::Percent => "1%",
        ValueType::Angle => "1deg",
        ValueType::Text => "\"validation\"",
        ValueType::Color => "#000000ff",
        ValueType::Boolean => "true",
        ValueType::Identifier => IDENTIFIER_BINDING,
    }
}

fn slot_source(kind: crate::program::model::SlotKind, names: Environment) -> String {
    use crate::program::model::SlotKind;
    match kind {
        SlotKind::Video => format!("source media resource {};", names.video),
        SlotKind::Audio => format!("source media resource {};", names.audio),
        SlotKind::Visual => "source generated transparent;".to_owned(),
        SlotKind::Text => text_source("text"),
        SlotKind::Caption => text_source("caption"),
        SlotKind::Sequence => format!("source sequence sequence {};", names.sequence),
    }
}

fn text_source(kind: &str) -> String {
    format!(
        "source {kind} {{ content \"validation\"; \
         style {{ font family \"Arial\"; size 16px; fill #ffffffff; }} \
         layout {{ box-width 1px; box-height 1px; wrap word; overflow clip; \
         horizontal-align center; vertical-align middle; }} }}"
    )
}
