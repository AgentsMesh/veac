use std::sync::{mpsc, Arc};
use std::time::Duration;

use super::*;
use crate::program::loader::{LoadedSource, SourceAuthority, SourceLoader};

const ROOT: &str = concat!(
    "module { import \"./leaf.veac\" as leaf; ",
    "export fn value() -> time { leaf.value() } }"
);
const LEAF: &str = "module { export fn value() -> time { 1s } }";
const CHANGED_LEAF: &str = "module { export fn value() -> time { 2s } }";

struct BarrierLoader {
    entered: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
}

impl SourceLoader for BarrierLoader {
    fn load(&self, _: &str, requested: &str) -> Result<LoadedSource, String> {
        assert_eq!(requested, "./leaf.veac");
        self.entered.send(()).unwrap();
        self.release.recv().unwrap();
        Ok(leaf(LEAF))
    }

    fn authority(&self, _: &str) -> SourceAuthority {
        SourceAuthority::Project
    }
}

#[test]
fn same_revision_barrier_readmits_writer_and_tracks_dependency_drift() {
    let database = Arc::new(CompilerDatabase::default());
    let worker_database = Arc::clone(&database);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        worker_database.prepare_source_graph(
            root(),
            &BarrierLoader {
                entered: entered_tx,
                release: release_rx,
            },
            &[],
        )
    });
    entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    database
        .parse_with_route_admission("root.veac", ROOT)
        .unwrap();
    release_tx.send(()).unwrap();
    let graph = worker.join().unwrap().unwrap();

    assert_eq!(
        graph.resolved_id("root.veac", "./leaf.veac"),
        Some("leaf.veac")
    );
    assert_registered(&database);
    database.parse("leaf.veac", CHANGED_LEAF).unwrap();
    assert_eq!(database.pending_invalidations(), ["root.veac"]);
}

fn assert_registered(database: &CompilerDatabase) {
    let snapshot = database.dependency_snapshot("root.veac").unwrap();
    assert_eq!(snapshot.direct_dependencies, ["leaf.veac"]);
    assert_eq!(snapshot.routes.len(), 1);
    assert_eq!(snapshot.routes[0].requested_path, "./leaf.veac");
    assert_eq!(snapshot.routes[0].resolved_source_id, "leaf.veac");
}

fn root() -> LoadedSource {
    LoadedSource {
        id: "root.veac".into(),
        source: ROOT.into(),
    }
}

fn leaf(source: &str) -> LoadedSource {
    LoadedSource {
        id: "leaf.veac".into(),
        source: source.into(),
    }
}
