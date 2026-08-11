use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

use super::*;

#[test]
fn panicked_initialization_releases_the_cache_for_retry() {
    let cache = DeadlineCache::<String>::default();
    let panicked = catch_unwind(AssertUnwindSafe(|| {
        let _: Result<String, DeadlineCacheError<()>> = cache
            .get_or_try_init(Instant::now() + Duration::from_secs(1), || {
                panic!("initialization failed")
            });
    }));
    assert!(panicked.is_err());

    let value = cache
        .get_or_try_init(Instant::now() + Duration::from_secs(1), || {
            Ok::<_, ()>("ready".to_owned())
        })
        .unwrap();
    assert_eq!(value, "ready");
}

#[test]
fn poisoned_state_is_reported_without_running_the_initializer() {
    let cache = DeadlineCache::<String>::default();
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _state = cache.state.lock().unwrap();
        panic!("poison cache state");
    }));

    let error = cache
        .get_or_try_init(Instant::now() + Duration::from_secs(1), || {
            Ok::<_, ()>("unused".to_owned())
        })
        .unwrap_err();
    assert!(matches!(error, DeadlineCacheError::Poisoned));
}
