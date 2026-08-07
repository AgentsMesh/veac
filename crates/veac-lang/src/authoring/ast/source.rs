super::define_syntax_tokens! {
    pub enum SourceKind {
        Media => "media",
        Text => "text",
        Caption => "caption",
        Generated => "generated",
        Sequence => "sequence",
        Multicam => "multicam",
    }
}
