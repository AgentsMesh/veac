use crate::authoring::{ApplyScopeKind, MappingKind};

use super::catalog::TermSet;
use super::GrammarPosition as P;

pub(super) const SETS: [TermSet; 2] = [
    TermSet::new(P::MappingKindPosition, MappingKind::TOKENS),
    TermSet::new(P::ApplyScopeKindPosition, ApplyScopeKind::TOKENS),
];
