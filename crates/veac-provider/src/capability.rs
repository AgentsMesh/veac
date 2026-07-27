use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::{ContentDigest, ProducerFingerprint};

use crate::validation::{invalid, text};
use crate::{ProviderResult, Validate, PROVIDER_SCHEMA_ID, PROVIDER_SCHEMA_VERSION};

mod negotiation;
pub use negotiation::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Asr,
    LanguageDetection,
    Translation,
    TextToSpeech,
    Dubbing,
    MotionTracking,
    Stabilization,
    Segmentation,
    Matte,
    Denoise,
    VocalSeparation,
    SceneDetection,
    BeatDetection,
    SilenceDetection,
    FillerDetection,
    HighlightDetection,
    AutoReframe,
    Retouch,
    Removal,
    ColorMatch,
    MulticamSync,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderFingerprint {
    pub provider: String,
    pub implementation_version: String,
    pub model: String,
    pub model_version: String,
    pub configuration: ContentDigest,
}

impl Validate for ProviderFingerprint {
    fn validate(&self) -> ProviderResult<()> {
        text(&self.provider, "provider name")?;
        text(
            &self.implementation_version,
            "provider implementation version",
        )?;
        text(&self.model, "provider model")?;
        text(&self.model_version, "provider model version")?;
        self.configuration.validate()?;
        Ok(())
    }
}

impl ProviderFingerprint {
    pub(crate) fn annotation_producer(&self) -> String {
        format!("{}/{}@{}", self.provider, self.model, self.model_version)
    }

    pub fn artifact_producer(&self) -> ProducerFingerprint {
        ProducerFingerprint {
            name: format!("{}/{}", self.provider, self.model),
            version: format!("{}+{}", self.implementation_version, self.model_version),
            configuration: self.configuration.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CapabilityOffer {
    pub capability: Capability,
    pub contract_versions: Vec<u32>,
    pub deterministic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderManifest {
    pub schema: String,
    pub schema_version: u32,
    pub fingerprint: ProviderFingerprint,
    pub offers: Vec<CapabilityOffer>,
}

impl ProviderManifest {
    pub fn new(fingerprint: ProviderFingerprint, offers: Vec<CapabilityOffer>) -> Self {
        Self {
            schema: PROVIDER_SCHEMA_ID.to_owned(),
            schema_version: PROVIDER_SCHEMA_VERSION,
            fingerprint,
            offers,
        }
    }
}

impl Validate for ProviderManifest {
    fn validate(&self) -> ProviderResult<()> {
        if self.schema != PROVIDER_SCHEMA_ID || self.schema_version != PROVIDER_SCHEMA_VERSION {
            return invalid("unsupported provider manifest schema");
        }
        self.fingerprint.validate()?;
        let mut previous = None;
        for offer in &self.offers {
            if previous.is_some_and(|value| value >= offer.capability) {
                return invalid("provider capabilities must be unique and sorted");
            }
            if offer.contract_versions.is_empty()
                || !offer.contract_versions.iter().all(|version| *version > 0)
                || !offer
                    .contract_versions
                    .windows(2)
                    .all(|pair| pair[0] < pair[1])
            {
                return invalid("capability contract versions must be positive and sorted");
            }
            previous = Some(offer.capability);
        }
        Ok(())
    }
}

pub fn canonical_provider_manifest_bytes(value: &ProviderManifest) -> ProviderResult<Vec<u8>> {
    value.validate()?;
    serde_json_canonicalizer::to_vec(value).map_err(Into::into)
}

pub fn provider_manifest_hash(value: &ProviderManifest) -> ProviderResult<ContentDigest> {
    canonical_provider_manifest_bytes(value).map(ContentDigest::sha256)
}
