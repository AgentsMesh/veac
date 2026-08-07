use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::VocabularyCategory;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalRole {
    BooleanLiteral,
    DeclarationIntroducer,
    DeclarationModifier,
    ReceiverBinding,
    #[schemars(skip)]
    FieldIntroducer,
    KindDiscriminator,
    ReferenceKind,
    ClauseIntroducer,
    #[schemars(skip)]
    InfixSeparator,
    #[schemars(skip)]
    DirectiveIntroducer,
    ClosedValue,
    PureFunction,
    NumericUnit,
    CompileTimeType,
}

impl CanonicalRole {
    pub const ALL: [Self; 11] = [
        Self::BooleanLiteral,
        Self::DeclarationIntroducer,
        Self::DeclarationModifier,
        Self::ReceiverBinding,
        Self::KindDiscriminator,
        Self::ReferenceKind,
        Self::ClauseIntroducer,
        Self::ClosedValue,
        Self::PureFunction,
        Self::NumericUnit,
        Self::CompileTimeType,
    ];

    pub(crate) const fn accepts(self, category: VocabularyCategory) -> bool {
        match category {
            VocabularyCategory::ReservedLiteral => matches!(self, Self::BooleanLiteral),
            VocabularyCategory::ContextualControl => matches!(
                self,
                Self::DeclarationIntroducer
                    | Self::DeclarationModifier
                    | Self::ReceiverBinding
                    | Self::FieldIntroducer
                    | Self::KindDiscriminator
                    | Self::ReferenceKind
                    | Self::ClauseIntroducer
                    | Self::InfixSeparator
                    | Self::DirectiveIntroducer
            ),
            VocabularyCategory::EnumValue => matches!(self, Self::ClosedValue),
            VocabularyCategory::BuiltinFunction => matches!(self, Self::PureFunction),
            VocabularyCategory::UnitSuffix => matches!(self, Self::NumericUnit),
            VocabularyCategory::ValueType => matches!(self, Self::CompileTimeType),
        }
    }
}
