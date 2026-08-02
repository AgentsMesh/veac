use super::Parser;
use crate::authoring::{ProjectSettings, Span};

impl Parser {
    pub(super) fn settings(&mut self) -> Option<ProjectSettings> {
        let start = self.required_word("settings")?;
        self.left_brace()?;
        let mut value = ProjectSettings::default();
        while !self.at_right_brace() && !self.at_eof() {
            let field = self.identifier("setting")?;
            match field.value.as_str() {
                "timebase" => {
                    let parsed = self.number("timebase");
                    self.semicolon();
                    set(self, "timebase", &mut value.timebase, parsed, field.span);
                }
                "canvas" => {
                    let width = self.number("canvas width");
                    self.required_word("by");
                    let height = self.number("canvas height");
                    self.semicolon();
                    let parsed = width.zip(height);
                    set(self, "canvas", &mut value.canvas, parsed, field.span);
                }
                "frame-rate" => {
                    let parsed = self.number("frame rate");
                    self.semicolon();
                    set(
                        self,
                        "frame-rate",
                        &mut value.frame_rate,
                        parsed,
                        field.span,
                    );
                }
                "sample-rate" => {
                    let parsed = self.number("audio sample rate");
                    self.semicolon();
                    set(
                        self,
                        "sample-rate",
                        &mut value.sample_rate,
                        parsed,
                        field.span,
                    );
                }
                _ => {
                    self.error(
                        "AUTHORING_SETTING_FIELD",
                        format!("unsupported project setting `{}`", field.value),
                        field.span,
                    );
                    self.recover_declaration();
                }
            }
        }
        let end = self.right_brace()?;
        value.span = start.join(end);
        Some(value)
    }
}

fn set<T>(
    parser: &mut Parser,
    name: &'static str,
    slot: &mut Option<T>,
    value: Option<T>,
    span: Span,
) {
    if slot.is_some() {
        parser.duplicate(name, span);
    } else if let Some(value) = value {
        *slot = Some(value);
    }
}
