mod api;
mod apply;
mod apply_ranges;
mod apply_target;
mod bounds;
mod build;
mod clip;
mod color;
mod components;
mod defaults;
mod effects;
mod error;
mod font;
mod material;
mod multicam;
mod output;
mod reachability;
mod relation_properties;
mod requirements;
mod resource;
mod selection;
mod source_time;
mod streams;
mod time;
mod timeline;
mod transition;

#[cfg(test)]
mod unit_tests;

use std::collections::{BTreeMap, BTreeSet};

use veac_ir::RelationGraph;

use veac_ir::{ProjectEnvelope, RenderConfig, SequenceId};

use crate::{ResolutionDiagnostic, ResolvedInput, ResolvedSequence};

pub use api::{resolve, resolve_one};
pub use requirements::required_material_ids_one;

pub(super) struct PlanResolver<'a> {
    envelope: &'a ProjectEnvelope,
    relations: RelationGraph<'a>,
    config: &'a RenderConfig,
    semantic_hash: &'a str,
    snapshot_hash: &'a str,
    reachability: reachability::Reachability,
    inputs: BTreeMap<crate::PlanInputId, ResolvedInput>,
    sequences: BTreeMap<SequenceId, ResolvedSequence>,
    sequence_order: Vec<SequenceId>,
    visiting: BTreeSet<SequenceId>,
    diagnostics: Vec<ResolutionDiagnostic>,
}

impl<'a> PlanResolver<'a> {
    fn new(
        envelope: &'a ProjectEnvelope,
        config: &'a RenderConfig,
        semantic_hash: &'a str,
        snapshot_hash: &'a str,
    ) -> Self {
        let reachability = reachability::Reachability::analyze(envelope, config);
        Self {
            envelope,
            relations: RelationGraph::project(&envelope.project),
            config,
            semantic_hash,
            snapshot_hash,
            reachability,
            inputs: BTreeMap::new(),
            sequences: BTreeMap::new(),
            sequence_order: Vec::new(),
            visiting: BTreeSet::new(),
            diagnostics: Vec::new(),
        }
    }
}
