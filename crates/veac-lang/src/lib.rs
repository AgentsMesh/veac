//! Agent-oriented executable VEAC language evaluated into canonical IR.

#[cfg(test)]
extern crate self as veac_lang;

mod authoring;
mod name;
pub mod program;
pub mod source_edit;
mod string_codec;
mod syntax_token;
pub mod vocabulary;

pub use syntax_token::SyntaxToken;
/// Exact compiler build identity for host-side computation cache contracts.
pub const COMPILER_BUILD_SHA256: &str = env!("VEAC_COMPILER_BUILD_SHA256");
pub(crate) use syntax_token::{
    define_syntax_tokens, impl_composite_syntax_tokens, impl_local_syntax_token_array_body,
    impl_local_syntax_token_body, impl_local_syntax_tokens, impl_syntax_tokens,
};
