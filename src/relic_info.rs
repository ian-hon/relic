use std::fs;

use serde::{Deserialize, Serialize};

use crate::{error::RelicError, paths::RELIC_PATH_INFO};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RelicInfo {
    pub remote: String,
    pub branch: String,
}
impl RelicInfo {
    pub fn empty() -> RelicInfo {
        RelicInfo {
            remote: "".to_string(),
            branch: "".to_string(),
        }
    }

    pub fn initialise() -> Result<RelicInfo, RelicError> {
        if let Ok(t) = fs::read_to_string(format!("./{RELIC_PATH_INFO}")) {
            if let Ok(d) = serde_json::from_str::<RelicInfo>(&t) {
                return Ok(d);
            }
            return Err(RelicError::RelicInfo(Box::new(
                RelicError::ConfigurationIncorrect,
            )));
        }
        Err(RelicError::RelicInfo(Box::new(RelicError::FileCantOpen)))
    }
}
