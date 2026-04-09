/// Media resolution pass: probes source files and enriches IR with metadata.
mod font;
mod resolver;

pub use font::resolve_font;
pub use resolver::*;
