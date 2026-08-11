//! Typed, deterministic visual evidence contracts and pure pixel evaluation.

mod authored;
mod bundle;
mod cache;
mod contract;
mod evaluate;
mod metrics;
mod model;
mod observation;
mod plan;
mod provenance;
mod report;
mod run;
mod schema;
mod validation;

pub use authored::*;
pub use bundle::{
    build_bundle_manifest, canonical_json, prepare_evidence_bundle, publish_evidence_bundle,
    sha256_hex, BundleError,
};
pub use cache::{CachedEvidenceBundle, EvidenceCache};
pub use contract::{decode_evidence_suite_json, EvidenceContractError};
pub use evaluate::evaluate;
pub use metrics::{
    alpha_stats, bounds, compare_composite, diff_stats, mask, motion_deceleration, resolve_region,
    reveal_prefix, source_over, AlphaStats, Bounds, CompositeStats, DiffStats, Mask, MetricError,
    MotionStats, PixelRect, RevealStats,
};
pub use model::*;
pub use observation::*;
pub use plan::*;
pub use provenance::*;
pub use report::*;
pub use run::*;
pub use schema::*;
pub use validation::{validate, ValidatedSuite, ValidationError};
pub use veac_ir::RationalTime;

pub fn authored_evidence_source_index_json_schema() -> serde_json::Value {
    veac_lang::program::source_index_json_schema()
        .expect("SourceIndexInventory schema is serializable")
}
