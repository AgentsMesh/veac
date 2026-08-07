mod use_macro;
mod use_model;
pub(crate) mod uses;
mod word;

pub use use_model::ControlUse;
pub use word::ControlWord;

pub(crate) fn all_uses() -> impl Iterator<Item = ControlUse> {
    uses::GROUPS.iter().flat_map(|group| group.iter().copied())
}

fn public_uses() -> impl Iterator<Item = ControlUse> {
    all_uses().filter(|usage| usage.layer().is_public())
}
