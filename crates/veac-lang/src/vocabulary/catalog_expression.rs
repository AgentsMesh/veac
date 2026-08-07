use super::catalog::TermSet;
use super::GrammarPosition;
use crate::program::expression::FunctionEffect;

pub(super) const SETS: [TermSet; 1] = [TermSet::new(
    GrammarPosition::ExpressionFunctionEffectValue,
    FunctionEffect::TOKENS,
)];
