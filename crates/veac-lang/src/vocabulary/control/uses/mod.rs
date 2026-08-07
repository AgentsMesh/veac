pub(crate) mod annotation;
pub(crate) mod apply;
pub(crate) mod audio;
pub(crate) mod color;
pub(crate) mod common;
pub(crate) mod delivery;
pub(crate) mod expression;
pub(crate) mod generator;
pub(crate) mod mapping;
pub(crate) mod modifier;
pub(crate) mod multicam;
pub(crate) mod parameter;
pub(crate) mod project;
pub(crate) mod relation;
pub(crate) mod resource;
pub(crate) mod static_program;
pub(crate) mod structural_type;
pub(crate) mod text;
pub(crate) mod timeline;

use super::ControlUse;

pub(super) const GROUPS: &[&[ControlUse]] = &[
    static_program::ALL,
    structural_type::ALL,
    expression::ALL,
    common::ALL,
    project::ALL,
    apply::ALL,
    mapping::ALL,
    relation::ALL,
    modifier::ALL,
    parameter::ALL,
    audio::ALL,
    color::ALL,
    resource::ALL,
    multicam::ALL,
    timeline::ALL,
    text::ALL,
    generator::ALL,
    delivery::ALL,
    annotation::ALL,
];
