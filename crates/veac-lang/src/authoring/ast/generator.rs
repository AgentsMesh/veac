super::define_syntax_tokens! {
    pub enum GeneratorKind {
        Transparent => "transparent",
        Silence => "silence",
        Solid => "solid",
        Gradient => "gradient",
        Shape => "shape",
    }
}
