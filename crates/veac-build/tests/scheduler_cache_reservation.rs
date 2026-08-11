mod support;

use veac_build::*;

#[derive(Clone, Copy)]
enum Mode {
    CancelBeforeWorker,
    CancelReservation,
    FailReservation,
    InvalidReservedHit,
}

struct ReservationCache {
    mode: Mode,
    cancellation: CancellationToken,
}

impl BuildCache for ReservationCache {
    fn get(&self, _key: &NodeCacheKey) -> BuildResult<Option<ArtifactOutputs>> {
        if matches!(self.mode, Mode::CancelBeforeWorker) {
            self.cancellation.cancel();
        }
        Ok(None)
    }

    fn put(&self, _key: &NodeCacheKey, _outputs: &ArtifactOutputs) -> BuildResult<()> {
        Ok(())
    }

    fn reserve(
        &self,
        _key: &NodeCacheKey,
        _cancellation: &CancellationToken,
    ) -> BuildResult<CacheReservation> {
        match self.mode {
            Mode::CancelReservation => Err(BuildError::cancelled("reservation cancelled")),
            Mode::FailReservation => Err(BuildError::cache("reservation failed")),
            Mode::InvalidReservedHit => Ok(CacheReservation::Hit(
                ArtifactOutputs::one(
                    &OutputSlot::<support::Media>::new("wrong").unwrap(),
                    ContentDigest::sha256(b"wrong"),
                )
                .unwrap(),
            )),
            Mode::CancelBeforeWorker => unreachable!(),
        }
    }
}

#[test]
fn worker_and_reservation_failures_keep_their_classification() {
    for (mode, status, message) in [
        (Mode::CancelBeforeWorker, NodeStatus::Cancelled, "cancelled"),
        (
            Mode::CancelReservation,
            NodeStatus::Cancelled,
            "reservation cancelled",
        ),
        (
            Mode::FailReservation,
            NodeStatus::Failed,
            "reservation failed",
        ),
        (
            Mode::InvalidReservedHit,
            NodeStatus::Failed,
            "reserved cache entry is invalid",
        ),
    ] {
        let cancellation = CancellationToken::new();
        let cache = ReservationCache {
            mode,
            cancellation: cancellation.clone(),
        };
        let mut graph = GraphBuilder::new();
        graph.add_node(support::node("node")).unwrap();
        let receipt = support::scheduler(1)
            .run(
                &graph.validate().unwrap(),
                &support::FakeExecutor::default(),
                &cache,
                cancellation,
            )
            .unwrap();
        assert_eq!(receipt.nodes[0].status, status);
        assert!(receipt.nodes[0]
            .message
            .as_deref()
            .unwrap()
            .contains(message));
    }
}
