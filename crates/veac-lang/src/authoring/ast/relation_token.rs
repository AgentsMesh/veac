pub use veac_ir::{
    CardinalDirection, CircleDirection, FadeColor, TrackMatteMode, TransitionAlignment,
    ZoomDirection,
};

super::impl_syntax_tokens!(TransitionAlignment,
    TransitionAlignment::Centered => "centered",
);
super::impl_syntax_tokens!(FadeColor,
    FadeColor::Transparent => "transparent",
    FadeColor::Black => "black",
    FadeColor::White => "white",
);
super::impl_syntax_tokens!(CardinalDirection,
    CardinalDirection::Left => "left",
    CardinalDirection::Right => "right",
    CardinalDirection::Up => "up",
    CardinalDirection::Down => "down",
);
super::impl_syntax_tokens!(ZoomDirection,
    ZoomDirection::In => "in",
    ZoomDirection::Out => "out",
);
super::impl_syntax_tokens!(CircleDirection,
    CircleDirection::Open => "open",
    CircleDirection::Close => "close",
);
super::impl_syntax_tokens!(TrackMatteMode,
    TrackMatteMode::Alpha => "alpha",
    TrackMatteMode::Luma => "luma",
);

super::define_syntax_tokens! {
    pub enum TransitionStyleKind {
        Dissolve => "dissolve",
        Fade => "fade",
        Wipe => "wipe",
        Slide => "slide",
        Zoom => "zoom",
        Circle => "circle",
        Pixelize => "pixelize",
    }
}
