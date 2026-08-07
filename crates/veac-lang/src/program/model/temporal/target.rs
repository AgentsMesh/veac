#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemporalItemPath {
    pub project: String,
    pub sequence: String,
    pub layer: String,
    pub item: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemporalApplyPath {
    pub project: String,
    pub sequence: String,
    pub apply: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TemporalTarget {
    Clip(TemporalItemPath),
    Text(TemporalItemPath),
    ClipMask {
        clip: TemporalItemPath,
        mask_index: u32,
    },
    ClipEffect {
        clip: TemporalItemPath,
        effect: String,
        parameter: String,
    },
    Apply(TemporalApplyPath),
    ApplyMask {
        apply: TemporalApplyPath,
        mask_index: u32,
    },
    ApplyEffect {
        apply: TemporalApplyPath,
        stage: String,
        effect: String,
        parameter: String,
    },
}

impl TemporalItemPath {
    pub(crate) fn segments(&self) -> [&str; 4] {
        [&self.project, &self.sequence, &self.layer, &self.item]
    }

    pub(crate) fn sequence_segments(&self) -> [&str; 2] {
        [&self.project, &self.sequence]
    }
}

impl TemporalApplyPath {
    pub(crate) fn segments(&self) -> [&str; 3] {
        [&self.project, &self.sequence, &self.apply]
    }

    pub(crate) fn sequence_segments(&self) -> [&str; 2] {
        [&self.project, &self.sequence]
    }
}

impl TemporalTarget {
    pub(crate) fn item(&self) -> Option<&TemporalItemPath> {
        match self {
            Self::Clip(path)
            | Self::Text(path)
            | Self::ClipMask { clip: path, .. }
            | Self::ClipEffect { clip: path, .. } => Some(path),
            Self::Apply(_) | Self::ApplyMask { .. } | Self::ApplyEffect { .. } => None,
        }
    }

    pub(crate) fn apply(&self) -> Option<&TemporalApplyPath> {
        match self {
            Self::Apply(path)
            | Self::ApplyMask { apply: path, .. }
            | Self::ApplyEffect { apply: path, .. } => Some(path),
            Self::Clip(_) | Self::Text(_) | Self::ClipMask { .. } | Self::ClipEffect { .. } => None,
        }
    }

    pub(crate) fn project(&self) -> &str {
        self.item().map_or_else(
            || &self.apply().expect("closed target").project,
            |path| &path.project,
        )
    }
}
