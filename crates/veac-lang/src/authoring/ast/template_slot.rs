super::define_syntax_tokens! {
    pub enum TemplateSlotKindDecl {
        Text => "text",
        Media => "media",
    }
}

super::define_syntax_tokens! {
    pub enum TemplateMediaKindDecl {
        Video => "video",
        Image => "image",
        VideoOrImage => "video-or-image",
    }
}

super::define_syntax_tokens! {
    pub enum TemplateFillDecl {
        FitDuration => "fit-duration",
        TakeHead => "take-head",
        TakeCenter => "take-center",
    }
}
