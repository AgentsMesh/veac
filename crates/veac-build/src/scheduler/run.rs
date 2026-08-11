use std::{
    collections::{BTreeMap, BTreeSet},
    panic::{catch_unwind, AssertUnwindSafe},
    sync::mpsc,
    thread,
};

use crate::{
    scheduler::{completion, prepare, state::RunState},
    ArtifactOutputs, BuildAction, BuildCache, BuildErrorKind, BuildLease, BuildLimits,
    BuildReceipt, BuildResult, CacheReservation, CancellationToken, ExecuteRequest, ExecutionError,
    NodeCacheKey, NodeExecutor, NodeId, NodeReceipt, NodeStatus, ValidatedGraph,
};

pub(super) struct Completion {
    pub(super) id: NodeId,
    pub(super) key: NodeCacheKey,
    pub(super) result: WorkerResult,
    pub(super) lease: Option<Box<dyn BuildLease>>,
}

pub(super) enum WorkerResult {
    CacheHit(ArtifactOutputs),
    Executed(Result<ArtifactOutputs, ExecutionError>),
}

pub(super) fn execute<A>(
    graph: &ValidatedGraph<A>,
    executor: &dyn NodeExecutor<A>,
    cache: &dyn BuildCache,
    cancellation: CancellationToken,
    limits: BuildLimits,
) -> BuildResult<BuildReceipt>
where
    A: BuildAction,
{
    super::state::validate_limits(graph, limits)?;
    let mut state = RunState::new(graph, limits);
    let mut prepared = BTreeMap::new();
    let mut cache_misses = BTreeSet::new();
    thread::scope(|scope| -> BuildResult<()> {
        let (sender, receiver) = mpsc::channel::<Completion>();
        loop {
            state.halted |= cancellation.is_cancelled();
            let mut progressed = false;
            if !state.halted {
                for id in state.ready(graph) {
                    if !prepared.contains_key(&id) {
                        let value = prepare::prepare(graph, executor, &id, &state.receipts)?;
                        prepared.insert(id.clone(), value);
                    }
                    let task = prepared.get(&id).expect("prepared node");
                    if !cache_misses.contains(&id) {
                        match cache.get(&task.key) {
                            Ok(Some(outputs)) => {
                                match prepare::validate_outputs(graph, &id, &outputs) {
                                    Ok(()) => state.record(NodeReceipt::new(
                                        id.clone(),
                                        NodeStatus::CacheHit,
                                        Some(task.key.clone()),
                                        Some(outputs),
                                        None,
                                    )),
                                    Err(error) => completion::fail(
                                        &mut state,
                                        &cancellation,
                                        id.clone(),
                                        Some(task.key.clone()),
                                        error.to_string(),
                                    ),
                                }
                                progressed = true;
                                break;
                            }
                            Ok(None) => {
                                cache_misses.insert(id.clone());
                            }
                            Err(error) => {
                                completion::fail(
                                    &mut state,
                                    &cancellation,
                                    id.clone(),
                                    Some(task.key.clone()),
                                    format!("cache lookup failed: {error}"),
                                );
                                progressed = true;
                                break;
                            }
                        }
                    }
                    let node = graph.node(&id).expect("validated node");
                    let claim = node.resources_claimed();
                    if !state.can_start(claim) {
                        continue;
                    }
                    let task = task.clone();
                    let sender = sender.clone();
                    let cancellation = cancellation.clone();
                    state.start(&id, claim);
                    scope.spawn(move || {
                        let result = catch_unwind(AssertUnwindSafe(|| {
                            if cancellation.is_cancelled() {
                                return (
                                    WorkerResult::Executed(Err(ExecutionError::cancelled(
                                        "build was cancelled",
                                    ))),
                                    None,
                                );
                            }
                            match cache.reserve(&task.key, &cancellation) {
                                Ok(CacheReservation::Hit(outputs)) => {
                                    (WorkerResult::CacheHit(outputs), None)
                                }
                                Ok(CacheReservation::Owner(lease)) => (
                                    WorkerResult::Executed(executor.execute(
                                        ExecuteRequest {
                                            node_id: &id,
                                            action: node.action(),
                                            inputs: &task.inputs,
                                            cache_key: &task.key,
                                        },
                                        &cancellation,
                                    )),
                                    Some(lease),
                                ),
                                Err(error) if error.kind() == BuildErrorKind::Cancelled => (
                                    WorkerResult::Executed(Err(ExecutionError::cancelled(
                                        error.to_string(),
                                    ))),
                                    None,
                                ),
                                Err(error) => (
                                    WorkerResult::Executed(Err(ExecutionError::failed(format!(
                                        "cache reservation failed: {error}"
                                    )))),
                                    None,
                                ),
                            }
                        }))
                        .unwrap_or_else(|_| {
                            (
                                WorkerResult::Executed(Err(ExecutionError::failed(
                                    "node executor panicked",
                                ))),
                                None,
                            )
                        });
                        let _ = sender.send(Completion {
                            id,
                            key: task.key,
                            result: result.0,
                            lease: result.1,
                        });
                    });
                    progressed = true;
                    break;
                }
            }
            if progressed {
                continue;
            }
            if state.running.is_empty() {
                break;
            }
            let completion = receiver
                .recv()
                .expect("a running build worker retains a channel sender");
            let receipt = completion::complete(graph, cache, completion);
            if matches!(receipt.status, NodeStatus::Failed | NodeStatus::Cancelled) {
                state.halted = true;
                cancellation.cancel();
            }
            state.record(receipt);
        }
        Ok(())
    })?;
    Ok(state.finish(graph, &prepared))
}
