use crate::{
    ResolvedRenderPlan, CAPABILITY_PROFILE, CURRENT_RENDER_PLAN_VERSION, EFFECT_REGISTRY_VERSION,
    RENDER_PLAN_SCHEMA_ID, RESOLVER_VERSION,
};

use super::{sha256_valid, Validator};

impl Validator {
    pub(super) fn header(&mut self, plan: &ResolvedRenderPlan) {
        let header = &plan.header;
        if header.schema != RENDER_PLAN_SCHEMA_ID
            || header.schema_version != CURRENT_RENDER_PLAN_VERSION
        {
            self.push(
                "PLAN_SCHEMA",
                "/header",
                "render plan schema identity or version is unsupported",
            );
        }
        self.source(plan);
        self.resolver(plan);
        self.cache(plan);
    }

    fn source(&mut self, plan: &ResolvedRenderPlan) {
        let source = &plan.header.source;
        if veac_ir::ProjectId::new(source.project_id.as_str()).is_err()
            || source.revision > veac_ir::MAX_SAFE_INTEGER
            || source.timebase == 0
        {
            self.push(
                "PLAN_SOURCE",
                "/header/source",
                "plan source identity, revision, or timebase is invalid",
            );
        }
        for (name, value) in [
            ("semantic_hash", &source.semantic_hash),
            ("snapshot_hash", &source.snapshot_hash),
        ] {
            if !sha256_valid(value) {
                self.push(
                    "PLAN_SOURCE_DIGEST",
                    format!("/header/source/{name}"),
                    "plan source digest must be lowercase SHA-256",
                );
            }
        }
        self.manifest(plan);
    }

    fn manifest(&mut self, plan: &ResolvedRenderPlan) {
        let value = &plan.header.source.executable;
        if value.core_version != veac_ir::CURRENT_CORE_VERSION
            || value.domain_opset_version != veac_ir::CURRENT_DOMAIN_OPSET_VERSION
            || value.temporal_opset_version != veac_ir::TEMPORAL_OPSET_VERSION
            || value.temporal_opset_version != plan.temporal.opset_version
            || !language_version_valid(&value.language_version)
        {
            self.push(
                "PLAN_EXECUTABLE_MANIFEST",
                "/header/source/executable",
                "executable manifest versions do not match the render plan contract",
            );
        }
        for digest in [
            &value.digests.domain_registry_sha256,
            &value.digests.main_core_sha256,
            &value.digests.source_graph_sha256,
            &value.digests.declared_inputs_sha256,
            &value.digests.compiler_sha256,
        ] {
            if !sha256_valid(digest) {
                self.push(
                    "PLAN_EXECUTABLE_DIGEST",
                    "/header/source/executable/digests",
                    "executable manifest digest must be lowercase SHA-256",
                );
            }
        }
    }

    fn resolver(&mut self, plan: &ResolvedRenderPlan) {
        let value = &plan.header.resolver;
        if value.resolver_version != RESOLVER_VERSION
            || value.effect_registry_version != EFFECT_REGISTRY_VERSION
            || value.capability_profile != CAPABILITY_PROFILE
            || value.stream_selection_policy.is_empty()
            || value.stream_selection_policy.len() > 256
            || value.stream_selection_policy.chars().any(char::is_control)
        {
            self.push(
                "PLAN_RESOLVER_IDENTITY",
                "/header/resolver",
                "resolver fingerprint is unsupported or malformed",
            );
        }
    }

    fn cache(&mut self, plan: &ResolvedRenderPlan) {
        let value = &plan.header.cache;
        let digests = [
            &value.project_semantic_sha256,
            &value.project_snapshot_sha256,
            &value.executable_manifest_sha256,
            &value.temporal_library_sha256,
            &value.resolver_sha256,
        ];
        if digests.into_iter().any(|digest| !sha256_valid(digest)) {
            self.push(
                "PLAN_CACHE_DIGEST",
                "/header/cache",
                "cache identities must be lowercase SHA-256",
            );
        }
        match crate::identity::cache(&plan.header.source, &plan.header.resolver, &plan.temporal) {
            Ok(expected) if expected == *value => {}
            _ => self.push(
                "PLAN_CACHE_IDENTITY",
                "/header/cache",
                "cache identity does not match source, resolver, and temporal content",
            ),
        }
    }
}

fn language_version_valid(value: &str) -> bool {
    let components = value.split('.').collect::<Vec<_>>();
    !value.is_empty()
        && value.len() <= 32
        && components.len() == 3
        && components.iter().all(|component| {
            !component.is_empty()
                && component.bytes().all(|byte| byte.is_ascii_digit())
                && (component.len() == 1 || !component.starts_with('0'))
        })
}
