use crate::{id::checked, BuildError, BuildResult, MAX_ACTION_BYTES};

pub trait BuildAction: Send + Sync + 'static {
    fn kind(&self) -> &str;
    fn version(&self) -> u32;
    fn canonical_bytes(&self) -> BuildResult<Vec<u8>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActionContract {
    pub kind: String,
    pub version: u32,
    pub bytes: Vec<u8>,
}

impl ActionContract {
    pub fn capture(action: &dyn BuildAction) -> BuildResult<Self> {
        let kind = checked(action.kind().to_owned(), 128, "action kind")?;
        if action.version() == 0 {
            return Err(BuildError::invalid("action version must be positive"));
        }
        let bytes = action.canonical_bytes()?;
        if bytes.len() > MAX_ACTION_BYTES {
            return Err(BuildError::resource_limit(
                "canonical action exceeds its byte budget",
            ));
        }
        Ok(Self {
            kind,
            version: action.version(),
            bytes,
        })
    }
}
