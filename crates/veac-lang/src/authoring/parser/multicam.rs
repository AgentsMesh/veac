use super::item::{assign, required};
use super::Parser;
use crate::authoring::{
    MulticamAngleDecl, MulticamDecl, MulticamSwitchDecl, MulticamSyncBasisDecl, MulticamSyncDecl,
    SourceDecl, Span, TypedReference,
};

impl Parser {
    pub(super) fn multicam(&mut self) -> Option<MulticamDecl> {
        let start = self.required_word("multicam")?;
        let id = self.identifier("multicam")?;
        self.left_brace()?;
        let mut sync = None;
        let mut angles = Vec::new();
        while !self.at_right_brace() && !self.at_eof() {
            if self.at_word("sync") {
                let field = self.current().span;
                let value = self.multicam_sync();
                assign(self, "multicam sync", &mut sync, value, field);
            } else if self.at_word("angle") {
                if let Some(value) = self.multicam_angle() {
                    angles.push(value);
                }
            } else {
                self.multicam_member_error();
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        Some(MulticamDecl {
            id,
            sync: required(self, "multicam sync", sync, span)?,
            angles,
            span,
        })
    }

    fn multicam_sync(&mut self) -> Option<MulticamSyncDecl> {
        let start = self.required_word("sync")?;
        let basis = self.identifier("multicam sync basis")?;
        let basis = match basis.value.as_str() {
            "timecode" => MulticamSyncBasisDecl::Timecode,
            "audio" => MulticamSyncBasisDecl::Audio,
            "manual" => MulticamSyncBasisDecl::Manual,
            _ => {
                self.error(
                    "AUTHORING_MULTICAM_SYNC",
                    "sync basis must be timecode, audio, or manual".to_owned(),
                    basis.span,
                );
                return None;
            }
        };
        self.left_brace()?;
        self.required_word("reference")?;
        let reference = self.reference_of_kind("angle")?;
        self.semicolon()?;
        let end = self.right_brace()?;
        Some(MulticamSyncDecl {
            basis,
            reference,
            span: start.join(end),
        })
    }

    fn multicam_angle(&mut self) -> Option<MulticamAngleDecl> {
        let start = self.required_word("angle")?;
        let id = self.identifier("multicam angle")?;
        self.left_brace()?;
        let mut resource = None;
        let mut offset = None;
        while !self.at_right_brace() && !self.at_eof() {
            let field = self.current().span;
            if self.at_word("source") {
                self.advance();
                let value = self.reference_of_kind("resource");
                self.semicolon();
                assign(self, "angle source", &mut resource, value, field);
            } else if self.at_word("source-offset") {
                self.advance();
                let value = self.number("multicam source offset");
                self.semicolon();
                assign(self, "angle source-offset", &mut offset, value, field);
            } else {
                self.error(
                    "AUTHORING_MULTICAM_ANGLE_FIELD",
                    "angle field must be source or source-offset".to_owned(),
                    field,
                );
                self.advance();
                self.recover_declaration();
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        Some(MulticamAngleDecl {
            id,
            resource: required(self, "angle source", resource, span)?,
            source_offset: required(self, "angle source-offset", offset, span)?,
            span,
        })
    }

    pub(super) fn multicam_source(&mut self, start: Span) -> Option<SourceDecl> {
        let group = self.reference_of_kind("multicam")?;
        self.left_brace()?;
        let mut switches = Vec::new();
        while !self.at_right_brace() && !self.at_eof() {
            if self.at_word("switch") {
                if let Some(value) = self.multicam_switch() {
                    switches.push(value);
                }
            } else {
                let span = self.current().span;
                self.error(
                    "AUTHORING_MULTICAM_SOURCE_MEMBER",
                    "multicam source members must be switches".to_owned(),
                    span,
                );
                self.advance();
                self.recover_declaration();
            }
        }
        let end = self.right_brace()?;
        Some(SourceDecl::Multicam {
            group,
            switches,
            span: start.join(end),
        })
    }

    fn multicam_switch(&mut self) -> Option<MulticamSwitchDecl> {
        let start = self.required_word("switch")?;
        let angle = self.reference_of_kind("angle")?;
        self.left_brace()?;
        let mut at = None;
        let mut duration = None;
        while !self.at_right_brace() && !self.at_eof() {
            let field = self.identifier("switch field")?;
            let value = self.number("switch time");
            self.semicolon();
            match field.value.as_str() {
                "at" => assign(self, "switch at", &mut at, value, field.span),
                "duration" => assign(self, "switch duration", &mut duration, value, field.span),
                _ => self.error(
                    "AUTHORING_MULTICAM_SWITCH_FIELD",
                    "switch field must be at or duration".to_owned(),
                    field.span,
                ),
            }
        }
        let end = self.right_brace()?;
        let span = start.join(end);
        Some(MulticamSwitchDecl {
            angle,
            at: required(self, "switch at", at, span)?,
            duration: required(self, "switch duration", duration, span)?,
            span,
        })
    }

    fn reference_of_kind(&mut self, expected: &'static str) -> Option<TypedReference> {
        let value = self.typed_reference()?;
        if value.kind.value == expected {
            Some(value)
        } else {
            self.error(
                "AUTHORING_REFERENCE_KIND",
                format!("reference must be `{expected} <id>`"),
                value.kind.span,
            );
            None
        }
    }

    fn multicam_member_error(&mut self) {
        let span = self.current().span;
        self.error(
            "AUTHORING_MULTICAM_MEMBER",
            "multicam member must be sync or angle".to_owned(),
            span,
        );
        self.advance();
        self.recover_declaration();
    }
}
