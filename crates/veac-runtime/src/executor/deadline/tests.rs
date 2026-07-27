use super::*;
use crate::RuntimeErrorKind;

#[test]
fn task_execution_limits_only_tighten_the_hard_policy() {
    assert!(TaskExecutionLimits::new(Duration::ZERO).is_err());
    assert!(TaskExecutionLimits::new(MAX_TASK_WALL_TIME + Duration::from_nanos(1)).is_err());
    let value = TaskExecutionLimits::new(Duration::from_millis(25)).unwrap();
    assert_eq!(value.max_wall_time(), Duration::from_millis(25));
    assert_eq!(
        TaskExecutionLimits::default().max_wall_time(),
        MAX_TASK_WALL_TIME
    );
    assert!(BundleSetupLimits::new(Duration::ZERO).is_err());
    assert!(BundleSetupLimits::new(MAX_TASK_WALL_TIME + Duration::from_nanos(1)).is_err());
    let setup = BundleSetupLimits::new(Duration::from_millis(50)).unwrap();
    assert_eq!(setup.max_wall_time(), Duration::from_millis(50));
    assert_eq!(
        BundleSetupLimits::default().max_wall_time(),
        MAX_TASK_WALL_TIME
    );
}

#[test]
fn expired_task_deadline_is_a_resource_limit() {
    let error = ensure(Instant::now()).unwrap_err();
    assert_eq!(error.kind, RuntimeErrorKind::ResourceLimit);
}
