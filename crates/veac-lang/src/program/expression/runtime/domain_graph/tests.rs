pub(super) use std::sync::Arc;

pub(super) use crate::program::{DomainOperationId, DomainOperationRegistry, DomainType};

pub(super) use super::super::super::{ExactNumber, PrimitiveType, Value, ValueType};
pub(super) use super::{DomainGraphTransaction, FrozenDomainGraph};

#[path = "tests/support.rs"]
mod support;
pub(super) use support::*;

#[path = "tests/construction.rs"]
mod construction;
#[path = "tests/freeze.rs"]
mod freeze;
#[path = "tests/plural_attachment.rs"]
mod plural_attachment;
#[path = "tests/refinement.rs"]
mod refinement;
#[path = "tests/relation.rs"]
mod relation;
#[path = "tests/resource.rs"]
mod resource;
#[path = "tests/topology.rs"]
mod topology;
#[path = "tests/transaction.rs"]
mod transaction;
#[path = "tests/update.rs"]
mod update;
