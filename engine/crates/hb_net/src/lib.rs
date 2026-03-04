









use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha384, Sha512};
use std::collections::BTreeMap;
use std::sync::Arc;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum NetError {
    #[error("network disabled/offline")]
    Offline,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("sri mismatch (expected {expected}, got {got})")]
    SRI { expected: String, got: String },
    #[error("invalid url: {0}")]
    Url(String),
    #[error("internal: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, NetError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SriAlg { Sha256, Sha384, Sha512 }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SRI {
    pub alg: SriAlg,
    pub b64: String,
}

#[derive(Debug, Clone)]
pub struct VerifiedBytes(pub Arc<[u8]>);

pub trait Loader: Send + Sync {
    fn get(&self, url: &Url, sri: Option<&SRI>) -> Result<VerifiedBytes>;
}


#[derive(Default)]
pub struct MemoryLoader {
    map: BTreeMap<String, Arc<[u8]>>,
}

impl MemoryLoader {
    pub fn new() -> Self { Self { map: BTreeMap::new() } }
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
        if got == sri.b64 { Ok(()) } else { Err(NetError::SRI { expected: sri.b64.clone(), got }) }
    }
}

impl Loader for MemoryLoader {
    fn get(&self, url: &Url, sri: Option<&SRI>) -> Result<VerifiedBytes> {
        let key = url.as_str().to_string();
        let bytes = self.map.get(&key).ok_or_else(|| NetError::NotFound(key.clone()))?.clone();
        if let Some(sri) = sri { Self::verify_sri(&bytes, sri)?; }
        Ok(VerifiedBytes(bytes))
    }
}


pub struct HttpClient {
    _allowed: bool,
}

impl HttpClient {
    pub fn disabled() -> Self { Self { _allowed: false } }
}

impl Loader for HttpClient {
    fn get(&self, _url: &Url, _sri: Option<&SRI>) -> Result<VerifiedBytes> {
        Err(NetError::Offline)
    }
}
