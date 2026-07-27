use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn material(&mut self, material: &Material, path: &str) {
        self.metadata(&material.metadata, path, material.id.as_str());
        let uri_valid = match &material.source {
            MaterialSource::File { uri } => project_relative_uri_valid(uri),
            MaterialSource::Remote { uri } => is_http_uri(uri),
        };
        if !uri_valid {
            self.push(
                "MATERIAL_URI",
                Some(material.id.to_string()),
                format!("{path}/source/uri"),
                "material URI is not a supported canonical locator",
                None,
            );
        }
        if material.identity.as_ref().is_some_and(|identity| {
            identity.algorithm != HashAlgorithm::Sha256
                || !super::values::is_sha256(&identity.digest)
        }) {
            self.value_error("MATERIAL_IDENTITY", path, material.id.as_str());
        }
        if !intent_valid(material.kind, &material.stream_intent) {
            self.value_error("STREAM_INTENT", path, material.id.as_str());
        }
        if let Some(probe) = &material.probe {
            if let Some(duration) = probe.container_duration {
                self.intrinsic_time(
                    duration,
                    true,
                    "PROBE_DURATION",
                    &format!("{path}/probe/duration"),
                    material.id.as_str(),
                );
            }
            self.probe(material, probe, path);
        }
    }
}

fn intent_valid(kind: MaterialKind, intent: &StreamIntent) -> bool {
    match kind {
        MaterialKind::Video => intent.video != StreamChoice::Disabled,
        MaterialKind::Image => {
            intent.video != StreamChoice::Disabled && intent.audio == StreamChoice::Disabled
        }
        MaterialKind::Audio => {
            intent.video == StreamChoice::Disabled && intent.audio != StreamChoice::Disabled
        }
        MaterialKind::Font | MaterialKind::Lut1d | MaterialKind::Lut3d => {
            intent.video == StreamChoice::Disabled && intent.audio == StreamChoice::Disabled
        }
    }
}

pub fn project_relative_uri_valid(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.contains(['\\', '\0'])
        && value
            .split('/')
            .all(|segment| !segment.is_empty() && !matches!(segment, "." | ".."))
        && !value
            .split('/')
            .next()
            .is_some_and(|segment| segment.ends_with(':'))
}

fn is_http_uri(value: &str) -> bool {
    value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
        .is_some_and(|remainder| {
            let authority = remainder.split('/').next().unwrap_or_default();
            !authority.is_empty()
                && !authority.chars().any(char::is_whitespace)
                && !value.chars().any(char::is_control)
        })
}
