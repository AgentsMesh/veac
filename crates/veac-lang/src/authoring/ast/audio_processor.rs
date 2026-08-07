super::define_syntax_tokens! {
    pub enum AudioProcessorKind {
        Eq => "eq",
        HighPass => "high-pass",
        LowPass => "low-pass",
        Compressor => "compressor",
        Limiter => "limiter",
        Gate => "gate",
        Loudness => "loudness",
    }
}
