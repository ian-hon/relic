use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RelicError {
    BlobCantOpen,
    IgnoredBlob,
    ConfigurationIncorrect,
    RelicInfo(Box<RelicError>),
}
