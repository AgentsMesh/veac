super::define_syntax_tokens! {
    pub enum MappingKind {
        Linear => "linear",
        Curve => "curve",
        Freeze => "freeze",
    }
}

super::impl_syntax_tokens!(veac_ir::SourceOutOfRangePolicy,
    veac_ir::SourceOutOfRangePolicy::Strict => "strict",
    veac_ir::SourceOutOfRangePolicy::HoldFirst => "hold-first",
    veac_ir::SourceOutOfRangePolicy::HoldLast => "hold-last",
    veac_ir::SourceOutOfRangePolicy::HoldBoth => "hold-both",
);
