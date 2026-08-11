use veac_lang::program::expression::Value;

use super::{kind, unknown_variant, Decoder};
use crate::authored::ProjectDecodeError;

impl<'a> Decoder<'a> {
    pub fn text(&self, value: &Value, path: &str) -> Result<String, ProjectDecodeError> {
        match value {
            Value::Text(value) => Ok(value.to_string()),
            _ => Err(kind(path, "text", value)),
        }
    }

    pub fn identifier(&self, value: &Value, path: &str) -> Result<String, ProjectDecodeError> {
        match value {
            Value::Identifier(value) => Ok(value.to_string()),
            _ => Err(kind(path, "identifier", value)),
        }
    }

    pub fn bool(&self, value: &Value, path: &str) -> Result<bool, ProjectDecodeError> {
        match value {
            Value::Bool(value) => Ok(*value),
            _ => Err(kind(path, "bool", value)),
        }
    }

    pub fn integer(&self, value: &Value, path: &str) -> Result<i64, ProjectDecodeError> {
        match value {
            Value::Integer(value) => Ok(*value),
            _ => Err(kind(path, "int", value)),
        }
    }

    pub fn u16(&self, value: &Value, path: &str) -> Result<u16, ProjectDecodeError> {
        u16::try_from(self.integer(value, path)?)
            .map_err(|_| ProjectDecodeError::new(path, "integer is outside u16 range"))
    }

    pub fn u8(&self, value: &Value, path: &str) -> Result<u8, ProjectDecodeError> {
        u8::try_from(self.integer(value, path)?)
            .map_err(|_| ProjectDecodeError::new(path, "integer is outside u8 range"))
    }

    pub fn u32(&self, value: &Value, path: &str) -> Result<u32, ProjectDecodeError> {
        u32::try_from(self.integer(value, path)?)
            .map_err(|_| ProjectDecodeError::new(path, "integer is outside u32 range"))
    }

    pub fn u64(&self, value: &Value, path: &str) -> Result<u64, ProjectDecodeError> {
        u64::try_from(self.integer(value, path)?)
            .map_err(|_| ProjectDecodeError::new(path, "integer is outside u64 range"))
    }

    pub fn optional_identifier(
        &'a self,
        value: &'a Value,
        path: &str,
    ) -> Result<Option<String>, ProjectDecodeError> {
        let variant = self.variant(value, "OptionalIdentifier", path)?;
        match variant.name {
            "None" => Ok(None),
            "Some" => self
                .identifier(variant.fields.get("value")?, &variant.fields.path("value"))
                .map(Some),
            name => Err(unknown_variant(path, "OptionalIdentifier", name)),
        }
    }

    pub fn optional_text(
        &'a self,
        value: &'a Value,
        path: &str,
    ) -> Result<Option<String>, ProjectDecodeError> {
        let variant = self.variant(value, "OptionalText", path)?;
        match variant.name {
            "None" => Ok(None),
            "Some" => self
                .text(variant.fields.get("value")?, &variant.fields.path("value"))
                .map(Some),
            name => Err(unknown_variant(path, "OptionalText", name)),
        }
    }
}
