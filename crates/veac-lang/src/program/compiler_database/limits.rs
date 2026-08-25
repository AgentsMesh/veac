#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompilerDatabaseLimits {
    pub syntax_entries: usize,
    pub syntax_source_bytes: usize,
    pub interface_entries: usize,
    pub interface_retained_bytes: usize,
    pub hir_entries: usize,
    pub hir_retained_bytes: usize,
    pub core_entries: usize,
    pub core_retained_bytes: usize,
    pub dependency_entries: usize,
    pub dependency_retained_bytes: usize,
}

impl Default for CompilerDatabaseLimits {
    fn default() -> Self {
        Self {
            syntax_entries: 4_096,
            syntax_source_bytes: 64 * 1024 * 1024,
            interface_entries: 1_024,
            interface_retained_bytes: 16 * 1024 * 1024,
            hir_entries: 4_096,
            hir_retained_bytes: 64 * 1024 * 1024,
            core_entries: 4_096,
            core_retained_bytes: 64 * 1024 * 1024,
            dependency_entries: 65_536,
            dependency_retained_bytes: 32 * 1024 * 1024,
        }
    }
}
