use std::fmt;

use veac_ir::ItemId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateErrorKind {
    InvalidRequest,
    InvalidProject,
    RevisionMismatch,
    NoTemplateTargets,
    MissingMediaBinding,
    UnexpectedMediaBinding,
    DuplicateMediaBinding,
    UnexpectedTextBinding,
    DuplicateTextBinding,
    InvalidText,
    MaterialIdMismatch,
    KindMismatch,
    MissingProbe,
    MissingIdentity,
    ProbeIdentityMismatch,
    MissingProbeFacts,
    MissingDuration,
    MediaTooShort,
    InexactTime,
    LockedTrack,
    EditRejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateError {
    pub kind: TemplateErrorKind,
    pub clip_id: Option<ItemId>,
    pub message: String,
}

impl TemplateError {
    pub(crate) fn new(kind: TemplateErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            clip_id: None,
            message: message.into(),
        }
    }

    pub(crate) fn clip(
        kind: TemplateErrorKind,
        clip_id: &ItemId,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            clip_id: Some(clip_id.clone()),
            message: message.into(),
        }
    }
}

impl fmt::Display for TemplateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.clip_id {
            Some(clip_id) => write!(
                formatter,
                "template error {:?} for {clip_id}: {}",
                self.kind, self.message
            ),
            None => write!(
                formatter,
                "template error {:?}: {}",
                self.kind, self.message
            ),
        }
    }
}

impl std::error::Error for TemplateError {}
