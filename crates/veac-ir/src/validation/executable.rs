use crate::*;

use super::{values::is_sha256, Validator};

impl Validator {
    pub(super) fn executable_manifest(
        &mut self,
        manifest: &ExecutableManifest,
        temporal: &TemporalProgramLibrary,
    ) {
        if !language_version_valid(&manifest.language_version) {
            self.value_error(
                "EXECUTABLE_LANGUAGE_VERSION",
                "/executable/language_version",
                "veac",
            );
        }
        for (actual, expected, code, pointer) in [
            (
                manifest.core_version,
                CURRENT_CORE_VERSION,
                "EXECUTABLE_CORE_VERSION",
                "/executable/core_version",
            ),
            (
                manifest.domain_opset_version,
                CURRENT_DOMAIN_OPSET_VERSION,
                "EXECUTABLE_DOMAIN_OPSET",
                "/executable/domain_opset_version",
            ),
            (
                manifest.temporal_opset_version,
                TEMPORAL_OPSET_VERSION,
                "EXECUTABLE_TEMPORAL_OPSET",
                "/executable/temporal_opset_version",
            ),
        ] {
            if actual != expected {
                self.value_error(code, pointer, "veac");
            }
        }
        if manifest.temporal_opset_version != temporal.opset_version {
            self.value_error(
                "EXECUTABLE_TEMPORAL_LIBRARY_OPSET",
                "/temporal/opset_version",
                "veac",
            );
        }
        let digests = &manifest.digests;
        for (name, value) in [
            ("domain_registry_sha256", &digests.domain_registry_sha256),
            ("main_core_sha256", &digests.main_core_sha256),
            ("source_graph_sha256", &digests.source_graph_sha256),
            ("declared_inputs_sha256", &digests.declared_inputs_sha256),
            ("compiler_sha256", &digests.compiler_sha256),
        ] {
            if !is_sha256(value) {
                self.value_error(
                    "EXECUTABLE_DIGEST",
                    &format!("/executable/digests/{name}"),
                    "veac",
                );
            }
        }
    }
}

fn language_version_valid(value: &str) -> bool {
    if value.is_empty() || value.len() > 32 {
        return false;
    }
    let components = value.split('.').collect::<Vec<_>>();
    components.len() == 3
        && components.iter().all(|component| {
            !component.is_empty()
                && component.bytes().all(|byte| byte.is_ascii_digit())
                && (component.len() == 1 || !component.starts_with('0'))
        })
}
