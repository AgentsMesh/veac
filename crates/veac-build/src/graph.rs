use std::collections::BTreeMap;

use veac_artifact::ContentDigest;

use crate::{
    action::ActionContract, BuildAction, BuildError, BuildResult, InputSlot, NodeId, OutputRef,
    OutputSlot, PortName, ResourceClaim,
};

mod digest;
mod validation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InputBinding {
    pub role: PortName,
    pub node: NodeId,
    pub output: PortName,
}

#[derive(Debug, Clone)]
pub struct NodeSpec<A> {
    pub(crate) id: NodeId,
    pub(crate) action: A,
    pub(crate) inputs: Vec<InputBinding>,
    pub(crate) order_after: Vec<NodeId>,
    pub(crate) outputs: Vec<PortName>,
    pub(crate) resources: ResourceClaim,
}

impl<A> NodeSpec<A> {
    pub fn new(id: NodeId, action: A) -> Self {
        Self {
            id,
            action,
            inputs: Vec::new(),
            order_after: Vec::new(),
            outputs: Vec::new(),
            resources: ResourceClaim::default(),
        }
    }

    pub fn output<T>(mut self, slot: &OutputSlot<T>) -> Self {
        self.outputs.push(slot.name().clone());
        self
    }

    pub fn bind<T>(mut self, slot: &InputSlot<T>, source: &OutputRef<T>) -> Self {
        self.inputs.push(InputBinding {
            role: slot.name().clone(),
            node: source.node.clone(),
            output: source.output.clone(),
        });
        self
    }

    pub fn resources(mut self, resources: ResourceClaim) -> Self {
        self.resources = resources;
        self
    }

    pub fn after(mut self, dependency: &NodeId) -> Self {
        self.order_after.push(dependency.clone());
        self
    }

    pub fn output_ref<T>(&self, slot: &OutputSlot<T>) -> OutputRef<T> {
        OutputRef {
            node: self.id.clone(),
            output: slot.name().clone(),
            marker: std::marker::PhantomData,
        }
    }

    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn action(&self) -> &A {
        &self.action
    }

    pub fn resources_claimed(&self) -> ResourceClaim {
        self.resources
    }

    pub fn output_names(&self) -> impl Iterator<Item = &PortName> {
        self.outputs.iter()
    }
}

#[derive(Debug, Clone)]
pub struct GraphBuilder<A> {
    nodes: BTreeMap<NodeId, NodeSpec<A>>,
}

impl<A> Default for GraphBuilder<A> {
    fn default() -> Self {
        Self {
            nodes: BTreeMap::new(),
        }
    }
}

impl<A> GraphBuilder<A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: NodeSpec<A>) -> BuildResult<()> {
        if self.nodes.contains_key(node.id()) {
            return Err(BuildError::invalid(format!(
                "node '{}' is declared more than once",
                node.id()
            )));
        }
        self.nodes.insert(node.id.clone(), node);
        Ok(())
    }
}

impl<A: BuildAction> GraphBuilder<A> {
    pub fn validate(self) -> BuildResult<ValidatedGraph<A>> {
        validation::validate(self.nodes)
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedGraph<A> {
    nodes: BTreeMap<NodeId, NodeSpec<A>>,
    actions: BTreeMap<NodeId, ActionContract>,
    topology: Vec<NodeId>,
    digest: ContentDigest,
}

impl<A> ValidatedGraph<A> {
    pub fn digest(&self) -> &ContentDigest {
        &self.digest
    }

    pub fn topology(&self) -> &[NodeId] {
        &self.topology
    }

    pub fn node(&self, id: &NodeId) -> Option<&NodeSpec<A>> {
        self.nodes.get(id)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub(crate) fn action_contract(&self, id: &NodeId) -> &ActionContract {
        &self.actions[id]
    }
}
