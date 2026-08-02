mod path;

use std::collections::BTreeMap;

use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::lexer;
use crate::program::token::TokenKind;

use super::budget::{self, Replacements};

pub(super) use path::InstancePath;

const PREFIX: &str = "veac-h-";

#[derive(Default)]
pub(super) struct Registry {
    generated: BTreeMap<String, InstancePath>,
    explicit: BTreeMap<String, Site>,
}

#[derive(Clone)]
struct Site {
    path: String,
    span: Span,
}

impl Registry {
    pub(super) fn reserve(
        &mut self,
        path: &str,
        logical: &InstancePath,
        span: Span,
    ) -> Result<String, Diagnostic> {
        if logical.is_root() {
            return logical.render(path, span);
        }
        let value = logical.render(path, span)?;
        if self
            .generated
            .get(&value)
            .is_some_and(|existing| existing != logical)
        {
            return Err(collision(path, &value, span));
        }
        if let Some(site) = self.explicit.get(&value) {
            return Err(collision(&site.path, &value, site.span));
        }
        self.generated.insert(value.clone(), logical.clone());
        Ok(value)
    }

    pub(super) fn source(&mut self, path: &str, source: &str) -> Result<(), Diagnostic> {
        let tokens = lexer::lex(path, source).map_err(|errors| errors[0].clone())?;
        for token in &tokens {
            if let TokenKind::Word(value) = &token.kind {
                self.explicit(path, value, token.span)?;
            }
        }
        Ok(())
    }

    pub(super) fn explicit(
        &mut self,
        path: &str,
        value: &str,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if !value.starts_with(PREFIX) {
            return Ok(());
        }
        if self.generated.contains_key(value) {
            return Err(collision(path, value, span));
        }
        self.explicit
            .entry(value.to_owned())
            .or_insert_with(|| Site {
                path: path.to_owned(),
                span,
            });
        Ok(())
    }

    fn generated(
        &mut self,
        path: &str,
        instance: &InstancePath,
        local: &str,
        span: Span,
    ) -> Result<String, Diagnostic> {
        self.reserve(path, &instance.child(local), span)
    }
}

pub(super) fn expand_with_limit(
    path: &str,
    source: String,
    instance: &InstancePath,
    registry: &mut Registry,
    limit: usize,
) -> Result<String, Diagnostic> {
    let tokens = lexer::lex(path, &source).map_err(|errors| errors[0].clone())?;
    for token in &tokens {
        if let TokenKind::Word(value) = &token.kind {
            registry.explicit(path, value, token.span)?;
        }
    }
    let mut local_ids = Vec::new();
    for token in tokens {
        let TokenKind::LocalId(local) = token.kind else {
            continue;
        };
        budget::ensure_replacement_available(path, local_ids.len())?;
        local_ids.push((token.span, local));
    }
    let spans = local_ids.iter().map(|(span, _)| span.start..span.end);
    let mut replacements = Replacements::with_owned(path, source, spans, limit)?;
    for (span, local) in local_ids {
        let value = registry.generated(path, instance, &local, span)?;
        replacements.push_owned(value)?;
    }
    replacements.finish()
}

pub(crate) fn encode(instance: &str, local: &str) -> String {
    InstancePath::root(instance).child(local).encoded()
}

pub(crate) fn encode_path(instance: &str, locals: &[&str]) -> String {
    locals
        .iter()
        .fold(InstancePath::root(instance), |path, local| {
            path.child(local)
        })
        .encoded()
}

fn collision(path: &str, value: &str, span: Span) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_HYGIENIC_ID_COLLISION",
        path,
        format!("generated component-local ID `{value}` collides with another ID"),
        span,
    )
}

#[cfg(test)]
mod tests;
