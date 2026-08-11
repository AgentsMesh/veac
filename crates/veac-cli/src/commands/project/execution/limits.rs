use veac_build::{BuildLimits, ResourceClaim};
use veac_project::{ExecutionPolicy, ProjectManifestV1};

use crate::error::{CliError, CliResult};

pub(super) fn from_manifest(manifest: &ProjectManifestV1) -> CliResult<BuildLimits> {
    let mut jobs = 1_usize;
    let mut cpu = 1_u16;
    let mut memory = 0_u32;
    let mut gpu = 0_u16;
    for profile in &manifest.profiles {
        match profile.execution {
            ExecutionPolicy::Serial {} => {}
            ExecutionPolicy::Parallel { max_tasks } => {
                jobs = jobs.max(usize::from(max_tasks));
                cpu = cpu.max(max_tasks);
            }
            ExecutionPolicy::ResourceAware {
                max_tasks,
                cpu_threads,
                memory_mib,
                gpu_slots,
            } => {
                jobs = jobs.max(usize::from(max_tasks));
                cpu = cpu.max(cpu_threads);
                memory = memory.max(memory_mib);
                gpu = gpu.max(gpu_slots);
            }
        }
    }
    match BuildLimits::new(jobs, ResourceClaim::new(cpu, memory, gpu)) {
        Ok(limits) => Ok(limits),
        Err(error) => Err(CliError::resource_limit(
            "PROJECT_BUILD_LIMIT",
            format!("invalid build limits: {error}"),
        )),
    }
}

#[cfg(test)]
#[path = "limits/tests.rs"]
mod tests;
