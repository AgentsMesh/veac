use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendFilterEscape {
    FilterValue,
    Quoted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendFilterBinding {
    File {
        token: String,
        path: PathBuf,
        escape: BackendFilterEscape,
    },
    Directory {
        token: String,
        directory: PathBuf,
        files: Vec<PathBuf>,
        escape: BackendFilterEscape,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendFilterContract {
    template: String,
    bindings: Vec<BackendFilterBinding>,
}

impl BackendFilterBinding {
    pub fn file(token: String, path: PathBuf, escape: BackendFilterEscape) -> Self {
        Self::File {
            token,
            path,
            escape,
        }
    }

    pub fn directory(
        token: String,
        directory: PathBuf,
        files: Vec<PathBuf>,
        escape: BackendFilterEscape,
    ) -> Self {
        Self::Directory {
            token,
            directory,
            files,
            escape,
        }
    }

    pub fn files(&self) -> &[PathBuf] {
        match self {
            Self::File { path, .. } => std::slice::from_ref(path),
            Self::Directory { files, .. } => files,
        }
    }

    fn token(&self) -> &str {
        match self {
            Self::File { token, .. } | Self::Directory { token, .. } => token,
        }
    }

    fn escape(&self) -> BackendFilterEscape {
        match self {
            Self::File { escape, .. } | Self::Directory { escape, .. } => *escape,
        }
    }

    fn original(&self) -> &Path {
        match self {
            Self::File { path, .. } => path,
            Self::Directory { directory, .. } => directory,
        }
    }
}

impl BackendFilterContract {
    pub fn new(template: String, bindings: Vec<BackendFilterBinding>) -> Result<Self, String> {
        let value = Self { template, bindings };
        value.validate_tokens()?;
        Ok(value)
    }

    pub fn bindings(&self) -> &[BackendFilterBinding] {
        &self.bindings
    }

    pub fn template(&self) -> &str {
        &self.template
    }

    pub fn render_original(&self) -> Result<String, String> {
        self.render(|binding| Some(binding.original().to_path_buf()))
    }

    pub fn render_bound(
        &self,
        files: &BTreeMap<PathBuf, PathBuf>,
        directories: &BTreeMap<Vec<PathBuf>, PathBuf>,
    ) -> Result<String, String> {
        self.render(|binding| match binding {
            BackendFilterBinding::File { path, .. } => files.get(path).cloned(),
            BackendFilterBinding::Directory { files, .. } => directories.get(files).cloned(),
        })
    }

    fn render(
        &self,
        mut resolve: impl FnMut(&BackendFilterBinding) -> Option<PathBuf>,
    ) -> Result<String, String> {
        self.validate_tokens()?;
        let mut replacements = Vec::with_capacity(self.bindings.len());
        for binding in &self.bindings {
            let path = resolve(binding)
                .ok_or_else(|| "filter resource has no verified path binding".to_owned())?;
            let text = path
                .to_str()
                .ok_or_else(|| "filter resource path is not valid UTF-8".to_owned())?;
            let start = self
                .template
                .find(binding.token())
                .ok_or_else(|| "filter resource token is absent from template".to_owned())?;
            replacements.push((start, binding.token().len(), escape(text, binding.escape())));
        }
        replacements.sort_unstable_by(|left, right| right.0.cmp(&left.0));
        let mut graph = self.template.clone();
        for (start, length, replacement) in replacements {
            graph.replace_range(start..start + length, &replacement);
        }
        Ok(graph)
    }

    fn validate_tokens(&self) -> Result<(), String> {
        token::validate(&self.template, &self.bindings)
    }
}

fn escape(value: &str, syntax: BackendFilterEscape) -> String {
    match syntax {
        BackendFilterEscape::Quoted => value
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace(':', "\\:")
            .replace(',', "\\,")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace(';', "\\;"),
        BackendFilterEscape::FilterValue => {
            let mut option = String::new();
            for character in value.chars() {
                if matches!(character, '\\' | '\'' | ':') {
                    option.push('\\');
                }
                option.push(character);
            }
            let mut graph = String::new();
            for character in option.chars() {
                if matches!(character, '\\' | '\'' | '[' | ']' | ',' | ';') {
                    graph.push('\\');
                }
                graph.push(character);
            }
            graph
        }
    }
}

#[cfg(test)]
mod tests;
