use super::super::use_macro::define_control_uses;

define_control_uses! {
    LET_BINDING => "let" @ ExpressionLetBinding : DeclarationIntroducer;
    MUTABLE_BINDING => "var" @ ExpressionMutableBinding : DeclarationIntroducer;
    MUTABLE_ASSIGNMENT => "set" @ ExpressionMutableAssignment : ClauseIntroducer;
    IF_BRANCH => "if" @ ExpressionIfBranch : ClauseIntroducer;
    ELSE_BRANCH => "else" @ ExpressionElseBranch : ClauseIntroducer;
    RANGE_STEP => "by" @ ExpressionRangeStep : ClauseIntroducer;
    CLOSURE => "fn" @ ExpressionClosureIntroducer : ClauseIntroducer;
    FUNCTION_EFFECT => "effect" @ ExpressionFunctionEffectClause : ClauseIntroducer;
    FOR_BINDING => "for" @ ExpressionForBinding : ClauseIntroducer;
    FOR_SOURCE => "in" @ ExpressionForSourceClause : ClauseIntroducer;
    MATCH_BRANCH => "match" @ ExpressionMatchIntroducer : ClauseIntroducer;
    TEMPORAL_ATTACHMENT => "animate" @ ExpressionTemporalAttachment : DirectiveIntroducer;
}
