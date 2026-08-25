use std::sync::{mpsc, Arc};
use std::time::Duration;

use super::*;

const SOURCE: &str =
    "fn main(context: Context) -> Project { project(identifier(\"p\"), project_settings(1)) }";

#[test]
fn statistics_is_an_atomic_barrier_before_a_waiting_query() {
    let database = Arc::new(CompilerDatabase::default());
    let statistics_database = Arc::clone(&database);
    let (admitted_tx, admitted_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (statistics_tx, statistics_rx) = mpsc::channel();
    let statistics_thread = std::thread::spawn(move || {
        statistics_tx
            .send(statistics_after_lifecycle_admission(
                &statistics_database,
                || {
                    admitted_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                },
            ))
            .unwrap();
    });
    admitted_rx.recv_timeout(Duration::from_secs(1)).unwrap();

    let query_database = Arc::clone(&database);
    let (query_tx, query_rx) = mpsc::channel();
    let query_thread = std::thread::spawn(move || {
        query_database.parse("waiting.veac", SOURCE).unwrap();
        query_tx.send(()).unwrap();
    });
    assert!(query_rx.recv_timeout(Duration::from_millis(50)).is_err());

    release_tx.send(()).unwrap();
    let snapshot = statistics_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    query_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    statistics_thread.join().unwrap();
    query_thread.join().unwrap();
    assert_eq!(snapshot.syntax_misses, 0);
    assert_eq!(snapshot.cached_syntax_entries, 0);
    assert_eq!(database.statistics().syntax_misses, 1);
    assert_eq!(database.statistics().cached_syntax_entries, 1);
}

fn statistics_after_lifecycle_admission(
    database: &CompilerDatabase,
    admitted: impl FnOnce(),
) -> CompilerDatabaseStatistics {
    let _lifecycle = database
        .lifecycle
        .write()
        .expect("compiler database lifecycle lock poisoned");
    admitted();
    database.collect_statistics()
}
