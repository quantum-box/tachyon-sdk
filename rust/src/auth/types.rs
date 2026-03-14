use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Macro to define a string-based ID newtype with common
/// trait implementations (Display, FromStr, Serialize,
/// Deserialize, etc.).
macro_rules! def_sdk_id {
    ($name:ident, $prefix:expr) => {
        #[derive(
            Clone, Eq, PartialEq, Hash, Ord, PartialOrd,
            Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Create from a raw string value.
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Return the inner string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Expected prefix for this ID type.
            pub fn prefix() -> &'static str {
                $prefix
            }
        }

        impl fmt::Debug for $name {
            fn fmt(
                &self,
                f: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(
                &self,
                f: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.to_string()))
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;
            fn deref(&self) -> &str {
                &self.0
            }
        }

        impl PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                self.0 == other
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.0 == *other
            }
        }

        impl PartialEq<String> for $name {
            fn eq(&self, other: &String) -> bool {
                &self.0 == other
            }
        }
    };
}

def_sdk_id!(TenantId, "tn_");
def_sdk_id!(UserId, "us_");
def_sdk_id!(ServiceAccountId, "sa_");
def_sdk_id!(PolicyId, "pol_");
def_sdk_id!(PublicApiKeyId, "pk_");

/// Platform identifier (alias for TenantId).
pub type PlatformId = TenantId;

/// Operator identifier (alias for TenantId).
pub type OperatorId = TenantId;

/// Validated identifier string (e.g. operator alias).
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Identifier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Identifier {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Public API key value (the actual key string).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PublicApiKeyValue(String);

impl PublicApiKeyValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PublicApiKeyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for PublicApiKeyValue {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}
