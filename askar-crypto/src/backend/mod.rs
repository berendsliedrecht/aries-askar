//! Supported key backends

use crate::Error;
use core::str::FromStr;

/// Backend of the key
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum KeyBackend {
    /// Software based keys
    #[default]
    Software,

    /// Keys generated and store in the secure element of the device
    SecureElement,
}

impl From<KeyBackend> for &str {
    fn from(key_backend: KeyBackend) -> Self {
        match key_backend {
            KeyBackend::Software => "software",
            KeyBackend::SecureElement => "secure_element",
        }
    }
}

impl FromStr for KeyBackend {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "software" => Ok(Self::Software),
            "secure_element" => Ok(Self::SecureElement),
            _ => Err(err_msg!(Invalid, "Invalid key backend.")),
        }
    }
}

impl core::fmt::Display for KeyBackend {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", <KeyBackend as Into<&str>>::into(self.clone()))
    }
}
