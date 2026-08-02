#[path = "tests/commit_deadline_cases.rs"]
mod commit_deadline_cases;
#[cfg(unix)]
#[path = "tests/pinning_cases.rs"]
mod pinning_cases;
#[cfg(unix)]
#[path = "tests/pinning_support.rs"]
mod pinning_support;
#[path = "tests/process_cases.rs"]
mod process_cases;
#[path = "tests/protocol_cases.rs"]
mod protocol_cases;
#[path = "tests/runner_cases.rs"]
mod runner_cases;
#[path = "tests/staging_cases.rs"]
mod staging_cases;
#[path = "tests/staging_deadline_cases.rs"]
mod staging_deadline_cases;
#[path = "tests/support.rs"]
mod support;
