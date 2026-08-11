use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectLiteral {
    Bool { value: bool },
    Integer { value: i64 },
    Scalar { value: ProjectRational },
    Text { value: String },
    Identifier { value: String },
    Duration { value: ProjectRational },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectRational {
    pub numerator: i64,
    pub denominator: u64,
}

impl ProjectRational {
    pub const fn new(numerator: i64, denominator: u64) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub fn is_canonical(self) -> bool {
        self.denominator != 0 && gcd(self.numerator.unsigned_abs(), self.denominator) == 1
    }
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}
