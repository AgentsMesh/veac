use crate::authoring::ModifierDecl;

pub(super) fn is_visual_domain(value: &ModifierDecl) -> bool {
    match value {
        ModifierDecl::Layout(_)
        | ModifierDecl::Transform(_)
        | ModifierDecl::Composite(_)
        | ModifierDecl::Surface(_)
        | ModifierDecl::Mask(_)
        | ModifierDecl::Color(_) => true,
        ModifierDecl::Audio(_) => false,
        ModifierDecl::Effect(value) => !matches!(
            veac_ir::effect_domain(&value.effect_type.value),
            Some(veac_ir::EffectDomain::Audio)
        ),
    }
}
