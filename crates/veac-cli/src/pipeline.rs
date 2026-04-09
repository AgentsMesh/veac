use std::path::Path;

use veac_lang::error::VeacError;
use veac_lang::ir::IrProgram;
use veac_lang::lexer::Lexer;
use veac_lang::parser::Parser;
use veac_lang::resolve::MediaResolver;
use veac_lang::semantic::SemanticAnalyzer;

use crate::cache::CachedProbeBackend;

/// Run the frontend pipeline: lex -> parse -> analyze.
/// Returns the validated IR on success.
pub fn compile(source: &str, file: &Path) -> Result<IrProgram, PipelineError> {
    let filename = file.display().to_string();
    let base_dir = file.parent().unwrap_or(Path::new("."));

    // Lexing
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| PipelineError {
        formatted: e.format(source, &filename),
        source_error: e,
    })?;

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| PipelineError {
        formatted: e.format(source, &filename),
        source_error: e,
    })?;

    // Semantic analysis
    let analyzer = SemanticAnalyzer::new(base_dir);
    let ir = analyzer.analyze(&program).map_err(|e| PipelineError {
        formatted: e.format(source, &filename),
        source_error: e,
    })?;

    Ok(ir)
}

/// Wraps a VeacError with its pre-formatted display string.
#[derive(Debug)]
pub struct PipelineError {
    pub formatted: String,
    #[allow(dead_code)]
    pub source_error: VeacError,
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.formatted)
    }
}

impl std::error::Error for PipelineError {}

/// Resolve media metadata on the IR using ffprobe with caching.
/// Cache is stored in `.veac-cache/` alongside the .veac source file.
/// Returns warnings for assets that could not be probed.
pub fn resolve_media(ir: &mut IrProgram, source_file: &Path) -> Vec<String> {
    let cache_dir = source_file
        .parent()
        .unwrap_or(Path::new("."))
        .join(".veac-cache");
    let backend = CachedProbeBackend::new(veac_runtime::asset::FfprobeBackend, &cache_dir);
    let resolver = MediaResolver::new(backend);
    let warnings = resolver.resolve(ir);
    warnings.into_iter().map(|w| w.message).collect()
}

/// Clean the probe cache for a given source file directory.
pub fn clean_cache(source_file: &Path) {
    let cache_dir = source_file
        .parent()
        .unwrap_or(Path::new("."))
        .join(".veac-cache");
    CachedProbeBackend::<veac_runtime::asset::FfprobeBackend>::clean(&cache_dir);
}
