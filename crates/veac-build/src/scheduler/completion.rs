use crate::{
    scheduler::{
        prepare,
        run::{Completion, WorkerResult},
        state::RunState,
    },
    BuildCache, CancellationToken, ExecutionErrorKind, NodeCacheKey, NodeId, NodeReceipt,
    NodeStatus, ValidatedGraph,
};

pub(super) fn complete<A>(
    graph: &ValidatedGraph<A>,
    cache: &dyn BuildCache,
    completion: Completion,
) -> NodeReceipt {
    let _lease = completion.lease;
    match completion.result {
        WorkerResult::CacheHit(outputs) => {
            match prepare::validate_outputs(graph, &completion.id, &outputs) {
                Ok(()) => NodeReceipt::new(
                    completion.id,
                    NodeStatus::CacheHit,
                    Some(completion.key),
                    Some(outputs),
                    None,
                ),
                Err(error) => NodeReceipt::new(
                    completion.id,
                    NodeStatus::Failed,
                    Some(completion.key),
                    None,
                    Some(format!("reserved cache entry is invalid: {error}")),
                ),
            }
        }
        WorkerResult::Executed(Ok(outputs)) => {
            let result = prepare::validate_outputs(graph, &completion.id, &outputs)
                .and_then(|()| cache.put(&completion.key, &outputs));
            match result {
                Ok(()) => NodeReceipt::new(
                    completion.id,
                    NodeStatus::Executed,
                    Some(completion.key),
                    Some(outputs),
                    None,
                ),
                Err(error) => NodeReceipt::new(
                    completion.id,
                    NodeStatus::Failed,
                    Some(completion.key),
                    None,
                    Some(format!("node publication failed: {error}")),
                ),
            }
        }
        WorkerResult::Executed(Err(error)) => {
            let status = match error.kind() {
                ExecutionErrorKind::Failed => NodeStatus::Failed,
                ExecutionErrorKind::Cancelled => NodeStatus::Cancelled,
            };
            NodeReceipt::new(
                completion.id,
                status,
                Some(completion.key),
                None,
                Some(error.message().to_owned()),
            )
        }
    }
}

pub(super) fn fail(
    state: &mut RunState,
    cancellation: &CancellationToken,
    id: NodeId,
    key: Option<NodeCacheKey>,
    message: String,
) {
    state.record(NodeReceipt::new(
        id,
        NodeStatus::Failed,
        key,
        None,
        Some(message),
    ));
    state.halted = true;
    cancellation.cancel();
}
