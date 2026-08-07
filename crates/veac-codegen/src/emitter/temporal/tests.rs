#[path = "tests/clocks.rs"]
mod clocks;
#[path = "tests/composites.rs"]
mod composites;
#[path = "tests/curves.rs"]
mod curves;
#[path = "tests/defense.rs"]
mod defense;
#[path = "tests/differential.rs"]
mod differential;
#[path = "tests/guards.rs"]
mod guards;
#[path = "tests/nodes.rs"]
mod nodes;
#[path = "tests/operations.rs"]
mod operations;
#[path = "tests/support.rs"]
pub(in crate::emitter) mod support;
#[path = "tests/typed_operations.rs"]
mod typed_operations;

use super::*;

const _: () = assert!(budget::MAX_EXPRESSION_BYTES < crate::emitter::MAX_INLINE_FILTER_GRAPH_BYTES);
const _: () = assert!(budget::MAX_COMPILED_BYTES > budget::MAX_EXPRESSION_BYTES);
