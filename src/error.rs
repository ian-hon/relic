use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RelicError {
    FileCantOpen,
    IgnoredFile,
    ConfigurationIncorrect
}