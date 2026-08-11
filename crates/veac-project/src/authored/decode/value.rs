use veac_lang::program::expression::{EnumValue, StructValue, Value};
use veac_lang::program::{
    EnumVariantDefinition, StructDefinition, TypeDefinition, TypeDefinitionKind, TypeRegistry,
};

use super::super::{ProjectDecodeError, PROJECT_MODULE_ID};

mod primitives;

pub(super) struct Decoder<'a> {
    types: &'a TypeRegistry,
}

pub(super) struct Fields<'a> {
    layout: &'a [veac_lang::program::FieldDefinition],
    values: &'a [Value],
    path: String,
}

pub(super) struct Variant<'a> {
    pub name: &'a str,
    pub fields: Fields<'a>,
}

impl<'a> Decoder<'a> {
    pub fn new(types: &'a TypeRegistry) -> Self {
        Self { types }
    }

    pub fn structure(
        &'a self,
        value: &'a Value,
        name: &str,
        path: &str,
    ) -> Result<Fields<'a>, ProjectDecodeError> {
        let Value::Struct(value) = value else {
            return Err(kind(path, "struct", value));
        };
        let definition = self.definition(value.as_ref(), name, path)?;
        let TypeDefinitionKind::Struct(layout) = definition.kind() else {
            return Err(ProjectDecodeError::new(
                path,
                format!("{name} is not a struct"),
            ));
        };
        Ok(fields(layout, value, path))
    }

    pub fn variant(
        &'a self,
        value: &'a Value,
        name: &str,
        path: &str,
    ) -> Result<Variant<'a>, ProjectDecodeError> {
        let Value::Enum(value) = value else {
            return Err(kind(path, "enum", value));
        };
        let definition = self.definition(value.as_ref(), name, path)?;
        let TypeDefinitionKind::Enum(layout) = definition.kind() else {
            return Err(ProjectDecodeError::new(
                path,
                format!("{name} is not an enum"),
            ));
        };
        let variant = layout
            .variants()
            .get(value.variant().index())
            .ok_or_else(|| {
                ProjectDecodeError::new(path, format!("{name} has an invalid variant index"))
            })?;
        Ok(Variant {
            name: variant.name(),
            fields: variant_fields(variant, value, path),
        })
    }

    pub fn list_map<T>(
        &'a self,
        value: &'a Value,
        path: &str,
        decode: fn(&Decoder<'_>, &Value, &str) -> Result<T, ProjectDecodeError>,
    ) -> Result<Vec<T>, ProjectDecodeError> {
        let Value::List(values) = value else {
            return Err(kind(path, "list", value));
        };
        values
            .values()
            .iter()
            .enumerate()
            .map(|(index, value)| decode(self, value, &format!("{path}[{index}]")))
            .collect()
    }

    fn definition<V: NominalValue>(
        &self,
        value: &V,
        name: &str,
        path: &str,
    ) -> Result<&'a TypeDefinition, ProjectDecodeError> {
        let definition = self.types.definition(value.type_id()).ok_or_else(|| {
            ProjectDecodeError::new(path, "nominal value type is absent from the registry")
        })?;
        if definition.canonical_source_id() != PROJECT_MODULE_ID
            || definition.declared_name() != name
            || definition.digest() != value.definition_digest()
        {
            return Err(ProjectDecodeError::new(
                path,
                format!(
                    "expected nominal {PROJECT_MODULE_ID}::{name}, got {}::{}",
                    definition.canonical_source_id(),
                    definition.declared_name()
                ),
            ));
        }
        Ok(definition)
    }
}

impl<'a> Fields<'a> {
    pub fn get(&self, name: &str) -> Result<&'a Value, ProjectDecodeError> {
        let field = self
            .layout
            .iter()
            .find(|field| field.name() == name)
            .ok_or_else(|| {
                ProjectDecodeError::new(self.path(name), "ABI field is missing from its definition")
            })?;
        self.values
            .get(field.index().index())
            .ok_or_else(|| ProjectDecodeError::new(self.path(name), "ABI field value is missing"))
    }

    pub fn path(&self, name: &str) -> String {
        format!("{}.{}", self.path, name)
    }
}

trait NominalValue {
    fn type_id(&self) -> veac_lang::program::TypeId;
    fn definition_digest(&self) -> veac_lang::program::TypeDefinitionDigest;
}

impl NominalValue for StructValue {
    fn type_id(&self) -> veac_lang::program::TypeId {
        self.type_id()
    }

    fn definition_digest(&self) -> veac_lang::program::TypeDefinitionDigest {
        self.definition_digest()
    }
}

impl NominalValue for EnumValue {
    fn type_id(&self) -> veac_lang::program::TypeId {
        self.type_id()
    }

    fn definition_digest(&self) -> veac_lang::program::TypeDefinitionDigest {
        self.definition_digest()
    }
}

fn fields<'a>(layout: &'a StructDefinition, value: &'a StructValue, path: &str) -> Fields<'a> {
    Fields {
        layout: layout.fields(),
        values: value.fields(),
        path: path.to_owned(),
    }
}

fn variant_fields<'a>(
    layout: &'a EnumVariantDefinition,
    value: &'a EnumValue,
    path: &str,
) -> Fields<'a> {
    Fields {
        layout: layout.fields(),
        values: value.fields(),
        path: path.to_owned(),
    }
}

pub(super) fn unknown_variant(path: &str, ty: &str, name: &str) -> ProjectDecodeError {
    ProjectDecodeError::new(path, format!("unknown {ty} variant `{name}`"))
}

fn kind(path: &str, expected: &str, value: &Value) -> ProjectDecodeError {
    ProjectDecodeError::new(
        path,
        format!("expected {expected}, got {:?}", value.value_type()),
    )
}
