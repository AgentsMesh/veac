use super::expression::{Effect, PrimitiveType, Stage};
use super::DomainType;

mod validation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryParameterContract {
    name: String,
    value_type: EntryValueType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryValueType {
    Primitive(PrimitiveType),
    Domain(DomainType),
    Nominal {
        source_id: Option<String>,
        declared_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryContract {
    function: String,
    parameters: Vec<EntryParameterContract>,
    result: EntryValueType,
    max_effect: Effect,
    max_stage: Stage,
    allow_inputs: bool,
    allow_temporal: bool,
    legacy_video: bool,
    preludes: Vec<String>,
}

impl EntryParameterContract {
    pub fn new(name: impl Into<String>, value_type: EntryValueType) -> Self {
        Self {
            name: name.into(),
            value_type,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value_type(&self) -> &EntryValueType {
        &self.value_type
    }
}

impl EntryValueType {
    pub fn nominal(declared_name: impl Into<String>) -> Self {
        Self::Nominal {
            source_id: None,
            declared_name: declared_name.into(),
        }
    }

    pub fn nominal_from(source_id: impl Into<String>, declared_name: impl Into<String>) -> Self {
        Self::Nominal {
            source_id: Some(source_id.into()),
            declared_name: declared_name.into(),
        }
    }
}

impl EntryContract {
    pub fn new(function: impl Into<String>, result: EntryValueType) -> Self {
        Self {
            function: function.into(),
            parameters: Vec::new(),
            result,
            max_effect: Effect::Pure,
            max_stage: Stage::Build,
            allow_inputs: false,
            allow_temporal: false,
            legacy_video: false,
            preludes: Vec::new(),
        }
    }

    pub fn video() -> Self {
        let mut value = Self::new("main", EntryValueType::Domain(DomainType::Project))
            .with_parameter(EntryParameterContract::new(
                "context",
                EntryValueType::Domain(DomainType::Context),
            ))
            .with_effect(Effect::GraphEmit)
            .with_stage(Stage::Temporal)
            .allow_inputs()
            .allow_temporal();
        value.legacy_video = true;
        value
    }

    pub fn with_parameter(mut self, value: EntryParameterContract) -> Self {
        self.parameters.push(value);
        self
    }

    pub fn with_effect(mut self, value: Effect) -> Self {
        self.max_effect = value;
        self
    }

    pub fn with_stage(mut self, value: Stage) -> Self {
        self.max_stage = value;
        self
    }

    pub fn allow_inputs(mut self) -> Self {
        self.allow_inputs = true;
        self
    }

    pub fn allow_temporal(mut self) -> Self {
        self.allow_temporal = true;
        self
    }

    pub fn with_prelude(mut self, source: impl Into<String>) -> Self {
        self.preludes.push(source.into());
        self
    }

    pub fn function(&self) -> &str {
        &self.function
    }

    pub fn parameters(&self) -> &[EntryParameterContract] {
        &self.parameters
    }

    pub fn result(&self) -> &EntryValueType {
        &self.result
    }

    pub fn preludes(&self) -> &[String] {
        &self.preludes
    }
}

#[cfg(test)]
mod tests;
