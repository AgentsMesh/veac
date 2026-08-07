#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FunctionEffect {
    Pure,
    Local,
    Emit,
    Any,
}

crate::impl_local_syntax_tokens!(
    FunctionEffect,
    FunctionEffect::Pure => "pure",
    FunctionEffect::Local => "local",
    FunctionEffect::Emit => "emit",
    FunctionEffect::Any => "any",
);

impl std::fmt::Display for FunctionEffect {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
