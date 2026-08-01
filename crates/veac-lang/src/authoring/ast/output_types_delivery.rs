#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GifDither {
    Bayer,
    FloydSteinberg,
    Sierra2,
    None,
}
