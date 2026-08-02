use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;

const PREFIX: &str = "veac-h-";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InstancePath {
    root: String,
    locals: Vec<String>,
}

impl InstancePath {
    pub(crate) fn root(id: &str) -> Self {
        Self {
            root: id.to_owned(),
            locals: Vec::new(),
        }
    }

    pub(crate) fn child(&self, local: &str) -> Self {
        let mut value = self.clone();
        value.locals.push(local.to_owned());
        value
    }

    pub(crate) fn render(&self, path: &str, span: Span) -> Result<String, Diagnostic> {
        let value = if self.locals.is_empty() {
            self.root.clone()
        } else {
            self.encoded()
        };
        if value.len() > 128 {
            return Err(Diagnostic::new(
                "PROGRAM_HYGIENIC_ID_LENGTH",
                path,
                "expanded component-local ID exceeds 128 bytes",
                span,
            ));
        }
        Ok(value)
    }

    pub(crate) fn encoded(&self) -> String {
        let mut value = format!("{PREFIX}{}-{}", self.root.len(), self.root);
        for local in &self.locals {
            value.push('-');
            value.push_str(&local.len().to_string());
            value.push('-');
            value.push_str(local);
        }
        value
    }

    pub(crate) fn is_root(&self) -> bool {
        self.locals.is_empty()
    }
}
