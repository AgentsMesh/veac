super::define_syntax_tokens! {
    pub enum ResourceKind {
        Video => "video",
        Audio => "audio",
        Image => "image",
        Font => "font",
        Lut1d => "lut-1d",
        Lut3d => "lut-3d",
    }
}

super::define_syntax_tokens! {
    pub enum ResourceLocatorKind {
        Local => "local",
        Remote => "remote",
    }
}

super::define_syntax_tokens! {
    pub enum ResourceIdentityKind {
        Sha256 => "sha256",
    }
}

super::define_syntax_tokens! {
    pub enum ResourceStreamSelection {
        Auto => "auto",
        Disabled => "disabled",
    }
}
