use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BuildLimits, BuildOutcome, BuildReceipt, NodeId, NodeReceipt, NodeStatus, ResourceClaim,
    ValidatedGraph,
};

pub(super) struct RunState {
    pub pending: BTreeSet<NodeId>,
    pub running: BTreeMap<NodeId, ResourceClaim>,
    pub receipts: BTreeMap<NodeId, NodeReceipt>,
    used: ResourceClaim,
    limits: BuildLimits,
    pub halted: bool,
}

impl RunState {
    pub fn new<A>(graph: &ValidatedGraph<A>, limits: BuildLimits) -> Self {
        Self {
            pending: graph.topology().iter().cloned().collect(),
            running: BTreeMap::new(),
            receipts: BTreeMap::new(),
            used: ResourceClaim::new(0, 0, 0),
            limits,
            halted: false,
        }
    }

    pub fn ready<A>(&self, graph: &ValidatedGraph<A>) -> Vec<NodeId> {
        self.pending
            .iter()
            .filter(|id| {
                graph
                    .node(id)
                    .expect("validated node")
                    .inputs
                    .iter()
                    .map(|input| &input.node)
                    .chain(graph.node(id).expect("validated node").order_after.iter())
                    .all(|dependency| {
                        self.receipts
                            .get(dependency)
                            .is_some_and(|receipt| receipt.status.is_success())
                    })
            })
            .cloned()
            .collect()
    }

    pub fn can_start(&self, claim: ResourceClaim) -> bool {
        self.running.len() < self.limits.jobs
            && claim.cpu_units <= self.limits.resources.cpu_units - self.used.cpu_units
            && claim.memory_mb <= self.limits.resources.memory_mb - self.used.memory_mb
            && claim.gpu_units <= self.limits.resources.gpu_units - self.used.gpu_units
    }

    pub fn start(&mut self, id: &NodeId, claim: ResourceClaim) {
        self.pending.remove(id);
        self.used.add(claim);
        self.running.insert(id.clone(), claim);
    }

    pub fn record(&mut self, receipt: NodeReceipt) {
        self.pending.remove(&receipt.node_id);
        if let Some(claim) = self.running.remove(&receipt.node_id) {
            self.used.subtract(claim);
        }
        self.receipts.insert(receipt.node_id.clone(), receipt);
    }

    pub fn finish<A>(
        mut self,
        graph: &ValidatedGraph<A>,
        prepared: &BTreeMap<NodeId, super::prepare::PreparedNode>,
    ) -> BuildReceipt {
        for id in graph.topology() {
            if !self.pending.contains(id) {
                continue;
            }
            let blocked = graph
                .node(id)
                .expect("validated node")
                .inputs
                .iter()
                .map(|input| &input.node)
                .chain(graph.node(id).expect("validated node").order_after.iter())
                .any(|dependency| {
                    self.receipts.get(dependency).is_some_and(|receipt| {
                        matches!(receipt.status, NodeStatus::Failed | NodeStatus::Blocked)
                    })
                });
            let status = if blocked {
                NodeStatus::Blocked
            } else {
                NodeStatus::Cancelled
            };
            let message = if blocked {
                "blocked by a failed dependency"
            } else {
                "build stopped before node admission"
            };
            let key = prepared.get(id).map(|value| value.key.clone());
            self.receipts.insert(
                id.clone(),
                NodeReceipt::new(id.clone(), status, key, None, Some(message.to_owned())),
            );
        }
        let outcome = if self
            .receipts
            .values()
            .any(|receipt| receipt.status == NodeStatus::Failed)
        {
            BuildOutcome::Failed
        } else if self
            .receipts
            .values()
            .any(|receipt| receipt.status == NodeStatus::Cancelled)
        {
            BuildOutcome::Cancelled
        } else {
            BuildOutcome::Succeeded
        };
        BuildReceipt {
            graph_digest: graph.digest().clone(),
            outcome,
            nodes: self.receipts.into_values().collect(),
        }
    }
}

pub(super) fn validate_limits<A>(
    graph: &ValidatedGraph<A>,
    limits: BuildLimits,
) -> crate::BuildResult<()> {
    for id in graph.topology() {
        let claim = graph.node(id).expect("validated node").resources_claimed();
        if !claim.fits(limits.resources) {
            return Err(crate::BuildError::resource_limit(format!(
                "node '{id}' requests more resources than the scheduler provides"
            )));
        }
    }
    Ok(())
}
