use serde::{Deserialize, Serialize};

use crate::store::StoreConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreNodeConfig {
    pub config: StoreConfig,
}
