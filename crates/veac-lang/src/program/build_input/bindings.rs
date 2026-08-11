use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use super::{BuildInputDeclaration, BuildInputManifestV1, BuildInputsError};
use crate::program::expression::{CoreBuildInputId, ResidualBuildBindings, Value, ValueLookup};
use crate::program::TypeRegistry;

pub(crate) struct VerifiedBuildInputs {
    values: BTreeMap<CoreBuildInputId, Value>,
    digest: Option<String>,
}

impl VerifiedBuildInputs {
    pub(crate) fn bind(
        declarations: &BTreeMap<String, BuildInputDeclaration>,
        types: &TypeRegistry,
        manifest: &BuildInputManifestV1,
    ) -> Result<Self, BuildInputsError> {
        manifest.validate_identity()?;
        let mut names = BTreeSet::new();
        let mut values = BTreeMap::new();
        for binding in &manifest.inputs {
            if !names.insert(binding.name.as_str()) {
                return Err(BuildInputsError::new(
                    "PROGRAM_INPUT_DUPLICATE",
                    format!("Build input `{}` is bound more than once", binding.name),
                ));
            }
            let declaration = declarations.get(&binding.name).ok_or_else(|| {
                BuildInputsError::new(
                    "PROGRAM_INPUT_UNKNOWN",
                    format!("unknown Build input `{}`", binding.name),
                )
            })?;
            if !binding.value.matches_declaration(
                declaration.role(),
                declaration.value_type(),
                types,
            ) {
                return Err(BuildInputsError::new(
                    "PROGRAM_INPUT_TYPE_MISMATCH",
                    format!(
                        "Build input `{}` expects {}, got {}",
                        binding.name,
                        declaration.value_type(),
                        binding.value.type_label()
                    ),
                ));
            }
            values.insert(
                declaration.id(),
                binding.value.bound_value(declaration.value_type(), types)?,
            );
        }
        if let Some(missing) = declarations
            .keys()
            .find(|name| !names.contains(name.as_str()))
        {
            return Err(BuildInputsError::new(
                "PROGRAM_INPUT_MISSING",
                format!("missing Build input `{missing}`"),
            ));
        }
        Ok(Self {
            digest: (!declarations.is_empty()).then(|| digest(declarations, &values)),
            values,
        })
    }

    pub(crate) fn digest(&self) -> Option<&str> {
        self.digest.as_deref()
    }

    pub(crate) fn residual_bindings(&self) -> &ResidualBuildBindings {
        &self.values
    }
}

impl ValueLookup for VerifiedBuildInputs {
    fn value(&self, _name: &str) -> Option<&Value> {
        None
    }

    fn build_value(&self, id: &CoreBuildInputId, _name: &str) -> Option<&Value> {
        self.values.get(id)
    }
}

fn digest(
    declarations: &BTreeMap<String, BuildInputDeclaration>,
    values: &BTreeMap<CoreBuildInputId, Value>,
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"veac.build-input-bindings-v1\0");
    for declaration in declarations.values() {
        digest.update(declaration.id().as_bytes());
        digest.update([declaration.role() as u8]);
        digest.update(declaration.value_type().to_string().as_bytes());
        digest_value(&mut digest, &values[&declaration.id()]);
    }
    format!("{:x}", digest.finalize())
}

fn digest_value(digest: &mut Sha256, value: &Value) {
    match value {
        Value::Bool(value) => digest.update([u8::from(*value)]),
        Value::Integer(value) => digest.update(value.to_be_bytes()),
        Value::Scalar(value) | Value::Time(value) | Value::Length(value) | Value::Angle(value) => {
            digest.update(value.numerator().to_be_bytes());
            digest.update(value.denominator().to_be_bytes());
        }
        Value::Text(value) | Value::Color(value) => {
            digest.update((value.len() as u64).to_be_bytes());
            digest.update(value.as_bytes());
        }
        Value::Enum(value) => {
            digest.update(value.type_id().as_bytes());
            digest.update(value.definition_digest().as_bytes());
            digest.update(value.variant().value().to_be_bytes());
        }
        Value::Struct(value) => {
            digest.update(value.type_id().as_bytes());
            digest.update(value.definition_digest().as_bytes());
            digest.update((value.fields().len() as u64).to_be_bytes());
            for field in value.fields() {
                digest_value(digest, field);
            }
        }
        _ => unreachable!("verified Build inputs are closed primitive or enum leaves"),
    }
}
