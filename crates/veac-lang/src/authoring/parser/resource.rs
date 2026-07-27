use super::item::{assign, required};
use super::resource_fields::{remote_identity, required_string};
use super::Parser;
use crate::authoring::{
    ResourceDecl, ResourceKind, ResourceLocator, ResourceStreams, StreamSelection,
};

impl Parser {
    pub(super) fn resource(&mut self) -> Option<ResourceDecl> {
        let start = self.required_word("resource")?;
        let kind = self.resource_kind()?;
        let id = self.identifier("resource")?;
        self.left_brace()?;
        let mut locator = None;
        let mut streams = None;
        while !self.at_right_brace() && !self.at_eof() {
            if self.at_word("locator") {
                let span = self.current().span;
                let value = self.resource_locator();
                assign(self, "resource locator", &mut locator, value, span);
            } else if self.at_word("streams") {
                let span = self.current().span;
                let value = self.resource_streams();
                assign(self, "resource streams", &mut streams, value, span);
            } else {
                let span = self.current().span;
                self.error(
                    "AUTHORING_RESOURCE_FIELD",
                    "resource field must be locator or streams".to_owned(),
                    span,
                );
                self.advance();
                self.recover_declaration();
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        if matches!(kind, ResourceKind::Video | ResourceKind::Audio) && streams.is_none() {
            self.error(
                "AUTHORING_REQUIRED_FIELD",
                "video and audio resources require streams".to_owned(),
                span,
            );
        }
        Some(ResourceDecl {
            kind,
            id,
            locator: required(self, "locator", locator, span)?,
            streams,
            span,
        })
    }

    fn resource_kind(&mut self) -> Option<ResourceKind> {
        let kind = self.identifier("resource type")?;
        let value = match kind.value.as_str() {
            "video" => ResourceKind::Video,
            "audio" => ResourceKind::Audio,
            "image" => ResourceKind::Image,
            "font" => ResourceKind::Font,
            "lut-1d" => ResourceKind::Lut1d,
            "lut-3d" => ResourceKind::Lut3d,
            _ => {
                self.error(
                    "AUTHORING_RESOURCE_TYPE",
                    "resource type must be video, audio, image, font, lut-1d, or lut-3d".to_owned(),
                    kind.span,
                );
                return None;
            }
        };
        Some(value)
    }

    fn resource_locator(&mut self) -> Option<ResourceLocator> {
        self.required_word("locator")?;
        let kind = self.identifier("locator type")?;
        let body = self.semantic_block()?;
        match kind.value.as_str() {
            "local" => Some(ResourceLocator::Local {
                path: required_string(self, &body, "path")?,
            }),
            "remote" => Some(ResourceLocator::Remote {
                uri: required_string(self, &body, "uri")?,
                identity: remote_identity(self, &body)?,
            }),
            _ => {
                self.error(
                    "AUTHORING_LOCATOR_TYPE",
                    "locator type must be local or remote".to_owned(),
                    kind.span,
                );
                None
            }
        }
    }

    fn resource_streams(&mut self) -> Option<ResourceStreams> {
        let start = self.required_word("streams")?;
        self.left_brace()?;
        let mut video = None;
        let mut audio = None;
        while !self.at_right_brace() && !self.at_eof() {
            let field = self.identifier("stream type")?;
            let value = self.stream_selection();
            self.semicolon();
            match field.value.as_str() {
                "video" => assign(self, "video stream", &mut video, value, field.span),
                "audio" => assign(self, "audio stream", &mut audio, value, field.span),
                _ => self.error(
                    "AUTHORING_STREAM_TYPE",
                    "stream type must be video or audio".to_owned(),
                    field.span,
                ),
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        Some(ResourceStreams {
            video: required(self, "video stream", video, span)?,
            audio: required(self, "audio stream", audio, span)?,
            span,
        })
    }

    fn stream_selection(&mut self) -> Option<StreamSelection> {
        let value = self.identifier("stream selection")?;
        match value.value.as_str() {
            "auto" => Some(StreamSelection::Auto),
            "disabled" => Some(StreamSelection::Disabled),
            _ => {
                self.error(
                    "AUTHORING_STREAM_SELECTION",
                    "stream selection must be auto or disabled".to_owned(),
                    value.span,
                );
                None
            }
        }
    }
}
