super::define_syntax_tokens! {
    pub enum ArtifactKind {
        Video => "video",
        ImageSequence => "image-sequence",
        CaptionSidecar => "caption-sidecar",
        AudioStem => "audio-stem",
        Scope => "scope",
        AudioFile => "audio-file",
        AnimatedImage => "animated-image",
        StillImage => "still-image",
        AdaptivePackage => "adaptive-package",
    }
}

super::define_syntax_tokens! {
    pub enum ArtifactTargetKind {
        File => "file",
        ImageSequence => "pattern",
        Package => "package",
    }
}
