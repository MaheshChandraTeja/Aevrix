use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha384, Sha512};
use std::collections::BTreeMap;
use std::sync::Arc;
use url::Url;

use crate::error::{EngineError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Origin {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
}

impl Origin {
    pub fn from_url(u: &Url) -> Option<Self> {
        Some(Self {
            scheme: u.scheme().to_string(),
            host: u.host_str()?.to_string(),
            port: u.port(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SriAlg {
    Sha256,
    Sha384,
    Sha512,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SRI {
    pub alg: SriAlg,
    pub b64: String,
}

#[derive(Debug, Clone)]
pub struct VerifiedBytes(pub Arc<[u8]>);

pub trait ResourceLoader: Send + Sync {
    fn get(&self, url: &Url, sri: Option<&SRI>) -> Result<VerifiedBytes>;
}

#[derive(Default)]
pub struct MemoryLoader {
    map: BTreeMap<String, Arc<[u8]>>,
}

impl MemoryLoader {
    pub fn new() -> Self {
        Self { map: BTreeMap::new() }
    }

    pub fn insert(&mut self, url: &str, bytes: Vec<u8>) {
        self.map.insert(url.to_string(), Arc::from(bytes.into_boxed_slice()));
    }

    fn verify_sri(bytes: &[u8], sri: &SRI) -> Result<()> {
        use base64::{engine::general_purpose::STANDARD, Engine};
        let got = match sri.alg {
            SriAlg::Sha256 => STANDARD.encode(Sha256::digest(bytes)),
            SriAlg::Sha384 => STANDARD.encode(Sha384::digest(bytes)),
            SriAlg::Sha512 => STANDARD.encode(Sha512::digest(bytes)),
        };
        if got == sri.b64 {
            Ok(())
        } else {
            Err(EngineError::SRI { expected: sri.b64.clone(), got })
        }
    }
}

impl ResourceLoader for MemoryLoader {
    fn get(&self, url: &Url, sri: Option<&SRI>) -> Result<VerifiedBytes> {
        let key = url.as_str().to_string();
        let bytes = self
            .map
            .get(&key)
            .ok_or_else(|| EngineError::Network(format!("not found: {}", key)))?
            .clone();

        if let Some(sri) = sri {
            Self::verify_sri(&bytes, sri)?;
        }
        Ok(VerifiedBytes(bytes))
    }
}
