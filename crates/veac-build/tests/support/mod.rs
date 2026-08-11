#![allow(dead_code)]

use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use veac_build::*;

pub struct Media;

pub fn input() -> InputSlot<Media> {
    InputSlot::new("input").unwrap()
}

pub fn output() -> OutputSlot<Media> {
    OutputSlot::new("artifact").unwrap()
}

pub fn id(value: &str) -> NodeId {
    NodeId::new(value).unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Behavior {
    Success,
    Fail,
    Cancel,
    Panic,
    WrongOutput,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub name: String,
    pub revision: u32,
    pub delay_ms: u64,
    pub observe_cancel: bool,
    pub behavior: Behavior,
}

impl Action {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            revision: 1,
            delay_ms: 0,
            observe_cancel: false,
            behavior: Behavior::Success,
        }
    }
}

impl BuildAction for Action {
    fn kind(&self) -> &str {
        "test-action"
    }

    fn version(&self) -> u32 {
        1
    }

    fn canonical_bytes(&self) -> BuildResult<Vec<u8>> {
        Ok(format!("{}:{}", self.name, self.revision).into_bytes())
    }
}

#[derive(Debug, Default)]
pub struct Stats {
    active: AtomicUsize,
    max_active: AtomicUsize,
    events: Mutex<Vec<String>>,
    counts: Mutex<BTreeMap<String, usize>>,
}

impl Stats {
    pub fn max_active(&self) -> usize {
        self.max_active.load(Ordering::SeqCst)
    }

    pub fn events(&self) -> Vec<String> {
        self.events.lock().unwrap().clone()
    }

    pub fn count(&self, name: &str) -> usize {
        self.counts.lock().unwrap().get(name).copied().unwrap_or(0)
    }

    fn start(&self, name: &str) {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_active.fetch_max(active, Ordering::SeqCst);
        self.events.lock().unwrap().push(format!("start:{name}"));
        *self
            .counts
            .lock()
            .unwrap()
            .entry(name.to_owned())
            .or_default() += 1;
    }

    fn finish(&self, name: &str) {
        self.events.lock().unwrap().push(format!("finish:{name}"));
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone)]
pub struct FakeExecutor {
    pub stats: Arc<Stats>,
    identity: ContentDigest,
}

impl Default for FakeExecutor {
    fn default() -> Self {
        Self {
            stats: Arc::default(),
            identity: ContentDigest::sha256(b"veac-build-test-executor-v1"),
        }
    }
}

impl FakeExecutor {
    pub fn with_identity(value: &[u8]) -> Self {
        Self {
            identity: ContentDigest::sha256(value),
            ..Self::default()
        }
    }

    pub fn with_digest(identity: ContentDigest) -> Self {
        Self {
            identity,
            ..Self::default()
        }
    }
}

impl NodeExecutor<Action> for FakeExecutor {
    fn implementation_identity(&self, _action: &Action) -> BuildResult<ContentDigest> {
        Ok(self.identity.clone())
    }

    fn execute(
        &self,
        request: ExecuteRequest<'_, Action>,
        cancellation: &CancellationToken,
    ) -> Result<ArtifactOutputs, ExecutionError> {
        let action = request.action;
        self.stats.start(&action.name);
        for _ in 0..action.delay_ms {
            thread::sleep(Duration::from_millis(1));
            if action.observe_cancel && cancellation.is_cancelled() {
                self.stats.finish(&action.name);
                return Err(ExecutionError::cancelled("observed cancellation"));
            }
        }
        let result = match action.behavior {
            Behavior::Success => success(action, request.inputs),
            Behavior::Fail => Err(ExecutionError::failed("requested failure")),
            Behavior::Cancel => Err(ExecutionError::cancelled("requested cancellation")),
            Behavior::Panic => panic!("requested panic"),
            Behavior::WrongOutput => ArtifactOutputs::one(
                &OutputSlot::<Media>::new("wrong").unwrap(),
                ContentDigest::sha256(b"wrong"),
            )
            .map_err(|error| ExecutionError::failed(error.to_string())),
        };
        self.stats.finish(&action.name);
        result
    }
}

fn success(action: &Action, inputs: &[ResolvedInput]) -> Result<ArtifactOutputs, ExecutionError> {
    let mut bytes = format!("{}:{}", action.name, action.revision).into_bytes();
    for input in inputs {
        bytes.extend_from_slice(input.role.as_str().as_bytes());
        bytes.extend_from_slice(input.digest.value.as_bytes());
    }
    ArtifactOutputs::one(&output(), ContentDigest::sha256(bytes))
        .map_err(|error| ExecutionError::failed(error.to_string()))
}

pub fn node(name: &str) -> NodeSpec<Action> {
    NodeSpec::new(id(name), Action::new(name)).output(&output())
}

pub fn scheduler(jobs: usize) -> BuildScheduler {
    BuildScheduler::new(BuildLimits::new(jobs, ResourceClaim::new(jobs as u16, 4096, 1)).unwrap())
}
