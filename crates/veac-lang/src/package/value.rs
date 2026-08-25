use std::{fmt, str::FromStr};

use schemars::JsonSchema;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

use super::{PackageError, PackageErrorKind};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct PackageName(
    #[schemars(
        length(min = 1, max = 128),
        regex(pattern = r"^[a-z][a-z0-9]*(?:[.-][a-z0-9]+)*$")
    )]
    String,
);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct ExactVersion(
    #[schemars(regex(
        pattern = r"^[0-9]+[.][0-9]+[.][0-9]+(?:-[0-9A-Za-z.-]+)?(?:[+][0-9A-Za-z.-]+)?$"
    ))]
    String,
);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct Sha256Digest(#[schemars(regex(pattern = r"^[0-9a-f]{64}$"))] String);

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PackageIdentity {
    pub name: PackageName,
    pub version: ExactVersion,
}

impl PackageName {
    pub fn new(value: impl Into<String>) -> Result<Self, PackageError> {
        let value = value.into();
        let valid = (1..=128).contains(&value.len())
            && value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
            })
            && value.split(['.', '-']).all(|part| !part.is_empty())
            && value.as_bytes().first().is_some_and(u8::is_ascii_lowercase);
        valid
            .then_some(Self(value))
            .ok_or_else(|| invalid("package name"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ExactVersion {
    pub fn new(value: impl Into<String>) -> Result<Self, PackageError> {
        let value = value.into();
        valid_semver(&value)
            .then_some(Self(value))
            .ok_or_else(|| invalid("exact semantic version"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Sha256Digest {
    pub fn from_bytes(value: [u8; 32]) -> Self {
        let mut encoded = String::with_capacity(64);
        for byte in value {
            use fmt::Write;
            write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
        }
        Self(encoded)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(value: &str) -> Result<Self, PackageError> {
        if value.len() != 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(invalid("lowercase SHA-256 digest"));
        }
        Ok(Self(value.to_owned()))
    }

    pub fn from_hex(value: &str) -> Result<Self, PackageError> {
        Self::parse(value)
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl fmt::Display for ExactVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl fmt::Display for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for PackageName {
    type Err = PackageError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}
impl FromStr for ExactVersion {
    type Err = PackageError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}
impl FromStr for Sha256Digest {
    type Err = PackageError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

macro_rules! deserialize_checked {
    ($type:ty, $constructor:expr) => {
        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                ($constructor)(String::deserialize(deserializer)?).map_err(D::Error::custom)
            }
        }
    };
}
deserialize_checked!(PackageName, PackageName::new);
deserialize_checked!(ExactVersion, ExactVersion::new);
deserialize_checked!(Sha256Digest, |value: String| Sha256Digest::parse(&value));

fn valid_semver(value: &str) -> bool {
    if value.len() > 128 {
        return false;
    }
    let (core_and_pre, build) = value
        .split_once('+')
        .map_or((value, None), |(core, build)| (core, Some(build)));
    if build.is_some_and(|part| part.is_empty() || part.contains('+')) {
        return false;
    }
    let (core, pre) = core_and_pre
        .split_once('-')
        .map_or((core_and_pre, None), |(core, pre)| (core, Some(pre)));
    let mut numbers = core.split('.');
    let valid_core =
        (0..3).all(|_| numbers.next().is_some_and(valid_number)) && numbers.next().is_none();
    valid_core
        && pre.is_none_or(|part| valid_identifiers(part, true))
        && build.is_none_or(|part| valid_identifiers(part, false))
}

fn valid_number(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}

fn valid_identifiers(value: &str, reject_leading_zero: bool) -> bool {
    !value.is_empty()
        && value.split('.').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && !(reject_leading_zero
                    && part.bytes().all(|byte| byte.is_ascii_digit())
                    && part.len() > 1
                    && part.starts_with('0'))
        })
}

fn invalid(label: &str) -> PackageError {
    PackageError::new(PackageErrorKind::Contract, format!("invalid {label}"))
}
