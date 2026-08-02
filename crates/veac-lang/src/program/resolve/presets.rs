use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::authoring::Span;
use crate::program::dependency_budget::DependencyBudget;
use crate::program::diagnostic::Diagnostic;
use crate::program::expand::{self, PresetUse};
use crate::program::model::{
    preset_key, preset_name_exists, PresetDecl, PresetKey, Scope, SurfaceFile,
};
use crate::program::parser::kind;

use super::retained;

pub(super) fn resolve(
    file: &SurfaceFile,
    scope: &mut Scope,
    definitions: &mut expand::definition::Budget,
    retained: &mut retained::Budget,
) -> Result<BTreeSet<PresetKey>, Diagnostic> {
    let mut declarations = BTreeMap::new();
    for declaration in &file.presets {
        let key = preset_key(declaration.kind, &declaration.name);
        if declarations.contains_key(&key) || scope.presets.contains_key(&key) {
            return Err(duplicate(file, declaration));
        }
        declarations.insert(key, declaration.clone());
    }
    let exported = declarations
        .iter()
        .filter(|(_, value)| value.exported)
        .map(|(key, _)| key.clone())
        .collect();
    let keys = declarations.keys().cloned().collect::<Vec<_>>();
    let mut resolver = Presets {
        file,
        declarations,
        scope,
        active: Vec::new(),
        budget: DependencyBudget::default(),
        definitions,
        retained,
    };
    for key in keys {
        resolver.body(&key)?;
    }
    Ok(exported)
}

struct Presets<'file, 'scope> {
    file: &'file SurfaceFile,
    declarations: BTreeMap<PresetKey, PresetDecl>,
    scope: &'scope mut Scope,
    active: Vec<PresetKey>,
    budget: DependencyBudget,
    definitions: &'scope mut expand::definition::Budget,
    retained: &'scope mut retained::Budget,
}

impl Presets<'_, '_> {
    fn body(&mut self, key: &PresetKey) -> Result<Arc<str>, Diagnostic> {
        if let Some(body) = self.scope.presets.get(key) {
            return Ok(Arc::clone(body));
        }
        if let Some(start) = self.active.iter().position(|value| value == key) {
            let mut chain = self.active[start..].iter().map(display).collect::<Vec<_>>();
            chain.push(display(key));
            return Err(Diagnostic::new(
                "PROGRAM_PRESET_CYCLE",
                &self.file.path,
                format!("preset dependency cycle: {}", chain.join(" -> ")),
                self.declarations[key].span,
            ));
        }
        let declaration = self.declarations[key].clone();
        self.budget
            .enter(&self.file.path, "preset", declaration.span)?;
        self.active.push(key.clone());
        let result = self.expand(&declaration);
        self.active.pop();
        self.budget.leave();
        let body = result?;
        expand::definition::preset(self.file, &declaration, &body, self.definitions)?;
        self.retained
            .preset(&self.file.path, &declaration.name, &body, declaration.span)?;
        let body: Arc<str> = Arc::from(body);
        Arc::make_mut(&mut self.scope.presets).insert(key.clone(), Arc::clone(&body));
        Ok(body)
    }

    fn expand(&mut self, declaration: &PresetDecl) -> Result<String, Diagnostic> {
        let start = declaration.body.content_span.start;
        let raw = &self.file.source[start..declaration.body.content_span.end];
        let uses =
            expand::preset_uses(&self.file.path, raw).map_err(|error| shift(error, start))?;
        for preset_use in uses {
            if preset_use.kind != declaration.kind {
                return Err(composition_kind_mismatch(
                    self.file,
                    declaration,
                    &preset_use,
                    start,
                ));
            }
            let key = preset_key(preset_use.kind, &preset_use.name);
            if self.declarations.contains_key(&key) {
                self.body(&key)?;
            } else if self.local_name_with_other_kind(&preset_use) {
                return Err(kind_mismatch(self.file, &preset_use, start));
            }
        }
        expand::preset_body(&self.file.path, raw, self.scope).map_err(|error| shift(error, start))
    }

    fn local_name_with_other_kind(&self, preset_use: &PresetUse) -> bool {
        preset_name_exists(&self.declarations, &preset_use.name)
    }
}

fn display(key: &PresetKey) -> String {
    format!("{} {}", kind::preset_name(key.1), key.0)
}

fn duplicate(file: &SurfaceFile, declaration: &PresetDecl) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_DUPLICATE_SYMBOL",
        &file.path,
        format!("preset `{}` is declared more than once", declaration.name),
        declaration.span,
    )
}

fn kind_mismatch(file: &SurfaceFile, value: &PresetUse, base: usize) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_PRESET_KIND_MISMATCH",
        &file.path,
        format!("preset `{}` has a different declared kind", value.name),
        offset(value.span, base),
    )
}

fn composition_kind_mismatch(
    file: &SurfaceFile,
    declaration: &PresetDecl,
    value: &PresetUse,
    base: usize,
) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_PRESET_KIND_MISMATCH",
        &file.path,
        format!(
            "{} preset `{}` cannot compose a {} preset",
            kind::preset_name(declaration.kind),
            declaration.name,
            kind::preset_name(value.kind)
        ),
        offset(value.span, base),
    )
}

fn shift(mut error: Diagnostic, base: usize) -> Diagnostic {
    error.span = offset(error.span, base);
    error
}

fn offset(span: Span, base: usize) -> Span {
    Span {
        start: span.start + base,
        end: span.end + base,
    }
}
