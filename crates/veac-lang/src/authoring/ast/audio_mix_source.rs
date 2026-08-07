#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioMixSourceKind {
    Master,
    Track,
    Bus,
}
