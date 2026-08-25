use sha2::{Digest, Sha256};

use super::super::{ExpressionContext, FunctionDefinition};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FunctionQueryKey([u8; 32]);

impl FunctionQueryKey {
    pub(crate) fn new(context: &ExpressionContext, definitions: &[FunctionDefinition]) -> Self {
        let mut digest = Sha256::new();
        digest.update(b"veac.function-query.v4\0");
        digest.update(crate::program::expression::CORE_VERSION.to_be_bytes());
        digest.update(crate::vocabulary::LANGUAGE_SPEC_SCHEMA_VERSION.to_be_bytes());
        digest.update(context.domain().version().raw().to_be_bytes());
        digest.update(context.domain().digest().as_bytes());
        length(&mut digest, definitions.len());
        for definition in definitions {
            frame(&mut digest, definition.name.as_bytes());
            frame(&mut digest, definition.body.as_bytes());
            super::super::core::encode_value_type(&mut digest, &definition.return_type);
            length(&mut digest, definition.parameters.len());
            for parameter in &definition.parameters {
                frame(&mut digest, parameter.name.as_bytes());
                super::super::core::encode_value_type(&mut digest, &parameter.value_type);
                optional_default(&mut digest, parameter.default());
            }
            origin_key(&mut digest, definition.origin.as_ref());
        }
        length(&mut digest, context.functions().namespaces().count());
        for namespace in context.functions().namespaces() {
            frame(&mut digest, namespace.as_bytes());
        }
        length(&mut digest, context.functions().visible_bindings().count());
        for (name, id) in context.functions().visible_bindings() {
            frame(&mut digest, name.as_bytes());
            digest.update(id.as_bytes());
        }
        length(
            &mut digest,
            context.functions().registry_functions().count(),
        );
        for (id, function) in context.functions().registry_functions() {
            digest.update(id.as_bytes());
            digest.update(function.content_digest().as_bytes());
            frame(&mut digest, function.name().as_bytes());
            origin_key(&mut digest, function.origin());
            length(&mut digest, function.parameters().len());
            for parameter in function.parameters() {
                frame(&mut digest, parameter.name.as_bytes());
                super::super::core::encode_value_type(&mut digest, &parameter.value_type);
                optional_default(&mut digest, parameter.default());
            }
            super::super::core::encode_value_type(&mut digest, function.return_type());
        }
        let build_inputs = context.build_input_bindings();
        length(&mut digest, build_inputs.len());
        for (binding, input) in build_inputs {
            frame(&mut digest, binding.as_bytes());
            frame(&mut digest, input.name().as_bytes());
            digest.update(input.id().as_bytes());
            super::super::core::encode_value_type(&mut digest, input.value_type());
        }
        length(&mut digest, context.methods().definitions().count());
        for method in context.methods().definitions() {
            digest.update(method.signature().function_id().as_bytes());
            if let Some(function) = context
                .functions()
                .lookup_by_id(method.signature().function_id())
            {
                digest.update([1]);
                digest.update(function.content_digest().as_bytes());
            } else {
                digest.update([0]);
            }
            length(
                &mut digest,
                method.signature().parameters_with_receiver().len(),
            );
            for parameter in method.signature().parameters_with_receiver() {
                frame(&mut digest, parameter.name.as_bytes());
                super::super::core::encode_value_type(&mut digest, &parameter.value_type);
                optional_default(&mut digest, parameter.default());
            }
            super::super::core::encode_value_type(&mut digest, method.signature().return_type());
            frame(&mut digest, method.owner_source().as_bytes());
            if let Some(body) = method.body() {
                digest.update([1]);
                frame(&mut digest, body.source().as_bytes());
                origin_key(&mut digest, Some(body.origin()));
            } else {
                digest.update([0]);
            }
        }
        length(&mut digest, context.types().definitions().len());
        for definition in context.types().definitions() {
            digest.update(definition.type_ref().id().as_bytes());
            digest.update(definition.digest().as_bytes());
        }
        length(&mut digest, context.types().names().len());
        for (name, reference) in context.types().names() {
            frame(&mut digest, name.as_bytes());
            digest.update(reference.id().as_bytes());
            frame(&mut digest, reference.diagnostic_name().as_bytes());
        }
        length(&mut digest, context.provisional_values().len());
        for (name, value_type) in context.provisional_values() {
            frame(&mut digest, name.as_bytes());
            super::super::core::encode_value_type(&mut digest, value_type);
        }
        Self(digest.finalize().into())
    }
}

fn frame(digest: &mut Sha256, value: &[u8]) {
    length(digest, value.len());
    digest.update(value);
}

fn length(digest: &mut Sha256, value: usize) {
    digest.update((value as u64).to_be_bytes());
}

fn optional_default(digest: &mut Sha256, default: Option<&super::super::FunctionDefault>) {
    if let Some(default) = default {
        digest.update([1]);
        default_key(digest, default);
    } else {
        digest.update([0]);
    }
}

fn default_key(digest: &mut Sha256, default: &super::super::FunctionDefault) {
    frame(digest, default.source().as_bytes());
    origin_key(digest, default.origin());
}

fn origin_key(digest: &mut Sha256, origin: Option<&super::super::FunctionOrigin>) {
    if let Some(origin) = origin {
        digest.update([1]);
        frame(digest, origin.source_id().as_bytes());
        let span = origin.body_span();
        digest.update((span.start as u64).to_be_bytes());
        digest.update((span.end as u64).to_be_bytes());
    } else {
        digest.update([0]);
    }
}

#[cfg(test)]
#[path = "query_key_semantic_tests.rs"]
mod semantic_tests;
#[cfg(test)]
#[path = "query_key_tests.rs"]
mod tests;
