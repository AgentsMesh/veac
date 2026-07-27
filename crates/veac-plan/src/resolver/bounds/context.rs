use veac_ir::{Clip, SourceOutOfRangePolicy};

#[derive(Clone, Copy)]
pub(in crate::resolver) struct BoundContext<'a> {
    pub(in crate::resolver) clip: &'a Clip,
    pub(in crate::resolver) path: &'a str,
    pub(in crate::resolver) outside: SourceOutOfRangePolicy,
}

impl<'a> BoundContext<'a> {
    pub(in crate::resolver) fn new(
        clip: &'a Clip,
        path: &'a str,
        outside: SourceOutOfRangePolicy,
    ) -> Self {
        Self {
            clip,
            path,
            outside,
        }
    }
}
