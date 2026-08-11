use crate::{BuildError, BuildResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceClaim {
    pub cpu_units: u16,
    pub memory_mb: u32,
    pub gpu_units: u16,
}

impl ResourceClaim {
    pub const fn new(cpu_units: u16, memory_mb: u32, gpu_units: u16) -> Self {
        Self {
            cpu_units,
            memory_mb,
            gpu_units,
        }
    }

    pub(crate) fn validate(self) -> BuildResult<()> {
        if self.cpu_units == 0 {
            Err(BuildError::invalid(
                "a node resource claim must reserve at least one CPU unit",
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn fits(self, capacity: Self) -> bool {
        self.cpu_units <= capacity.cpu_units
            && self.memory_mb <= capacity.memory_mb
            && self.gpu_units <= capacity.gpu_units
    }

    pub(crate) fn add(&mut self, value: Self) {
        self.cpu_units += value.cpu_units;
        self.memory_mb += value.memory_mb;
        self.gpu_units += value.gpu_units;
    }

    pub(crate) fn subtract(&mut self, value: Self) {
        self.cpu_units -= value.cpu_units;
        self.memory_mb -= value.memory_mb;
        self.gpu_units -= value.gpu_units;
    }
}

impl Default for ResourceClaim {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildLimits {
    pub jobs: usize,
    pub resources: ResourceClaim,
}

impl BuildLimits {
    pub fn new(jobs: usize, resources: ResourceClaim) -> BuildResult<Self> {
        if jobs == 0 || resources.cpu_units == 0 {
            return Err(BuildError::resource_limit(
                "build limits require positive jobs and CPU capacity",
            ));
        }
        Ok(Self { jobs, resources })
    }
}
