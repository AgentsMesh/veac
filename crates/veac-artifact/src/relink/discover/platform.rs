#[path = "platform/unix.rs"]
mod implementation;

pub(super) use implementation::{bind_roots, BoundDirectory, BoundEntry};
