use veac_plan::{ResolvedApply, ResolvedClip};

#[derive(Clone, Copy)]
pub(crate) enum ProcessOwner<'a> {
    Clip(&'a ResolvedClip),
    Apply(&'a ResolvedApply),
}

impl<'a> ProcessOwner<'a> {
    pub(crate) fn clip(value: &'a ResolvedClip) -> Self {
        Self::Clip(value)
    }

    pub(crate) fn apply(value: &'a ResolvedApply) -> Self {
        Self::Apply(value)
    }

    pub(super) fn id(self) -> String {
        match self {
            Self::Clip(clip) => clip.id.to_string(),
            Self::Apply(apply) => apply.id.to_string(),
        }
    }

    pub(super) fn kind(self) -> &'static str {
        match self {
            Self::Clip(_) => "clip",
            Self::Apply(_) => "apply",
        }
    }

    pub(super) fn as_clip(self) -> Option<&'a ResolvedClip> {
        match self {
            Self::Clip(clip) => Some(clip),
            Self::Apply(_) => None,
        }
    }
}
