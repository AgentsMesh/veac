mod identity;
mod model;
mod operations;

pub(in crate::executor) use model::{EntryIdentity, EntryState};

pub(super) use identity::{from_stat, opened_identity};

#[cfg(test)]
use identity::{identity_number, nonnegative_size};

#[cfg(test)]
#[path = "entry/tests.rs"]
mod tests;
