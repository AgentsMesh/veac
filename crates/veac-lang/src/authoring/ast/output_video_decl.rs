super::define_syntax_tokens! {
    pub enum MuxLayout {
        Standard => "standard",
        FastStart => "fast-start",
    }
}

super::define_syntax_tokens! {
    pub enum OutputChannelLayout {
        Mono => "mono",
        Stereo => "stereo",
        Discrete3 => "discrete-3",
        Discrete4 => "discrete-4",
        Discrete5 => "discrete-5",
        Surround51 => "surround-5-1",
        Discrete7 => "discrete-7",
        Surround71 => "surround-7-1",
    }
}

super::define_syntax_tokens! {
    pub enum AudioChannelLayoutDecl {
        Mono => "mono",
        Stereo => "stereo",
    }
}

super::define_syntax_tokens! {
    pub enum VideoRateControlKind {
        Crf => "crf",
        Average => "average",
        Capped => "capped",
        Lossless => "lossless",
    }
}

super::define_syntax_tokens! {
    pub enum GifPlaybackMode {
        Once => "once",
        Forever => "forever",
    }
}
