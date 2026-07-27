use super::item::{assign, required};
use super::Parser;
use crate::authoring::{Document, ProjectDecl, ProjectSettings};

impl Parser {
    pub(super) fn project(&mut self) -> Option<Document> {
        let start = self.required_word("project")?;
        let id = self.identifier("project")?;
        self.left_brace()?;
        let mut entry = None;
        let mut settings = None;
        let mut resources = Vec::new();
        let mut multicams = Vec::new();
        let mut sequences = Vec::new();
        let mut outputs = Vec::new();
        let mut annotations = Vec::new();
        while !self.at_right_brace() && !self.at_eof() {
            if self.at_word("entry") {
                let field = self.required_word("entry")?;
                let value = self.typed_reference();
                self.semicolon();
                let value = value.and_then(|value| {
                    if value.kind.value != "sequence" {
                        self.error(
                            "AUTHORING_ENTRY_KIND",
                            "project entry must be `entry sequence <id>;`".to_owned(),
                            value.kind.span,
                        );
                        None
                    } else {
                        Some(value)
                    }
                });
                assign(self, "project entry", &mut entry, value, field);
            } else if self.at_word("settings") {
                let value = self.settings();
                if settings.is_some() {
                    self.duplicate("settings", self.previous_span());
                } else {
                    settings = value;
                }
            } else if self.at_word("resource") {
                if let Some(value) = self.resource() {
                    resources.push(value);
                }
            } else if self.at_word("multicam") {
                if let Some(value) = self.multicam() {
                    multicams.push(value);
                }
            } else if self.at_word("sequence") {
                if let Some(value) = self.sequence() {
                    sequences.push(value);
                }
            } else if self.at_word("output") {
                if let Some(value) = self.output() {
                    outputs.push(value);
                }
            } else if self.at_word("annotation") {
                if let Some(value) = self.annotation() {
                    annotations.push(value);
                }
            } else {
                let span = self.current().span;
                self.error(
                    "AUTHORING_PROJECT_MEMBER",
                    "project member must be entry, settings, resource, multicam, sequence, annotation, or output"
                        .to_owned(),
                    span,
                );
                self.advance();
                self.recover_declaration();
            }
        }
        let end = self.right_brace()?;
        let settings = settings.unwrap_or_else(|| {
            self.error(
                "AUTHORING_REQUIRED_SETTINGS",
                "project requires a settings block".to_owned(),
                start.join(end),
            );
            ProjectSettings::default()
        });
        Some(Document {
            project: ProjectDecl {
                id,
                entry: required(self, "entry", entry, start.join(end))?,
                settings,
                resources,
                multicams,
                sequences,
                outputs,
                annotations,
                span: start.join(end),
            },
        })
    }
}
