use serde::{Deserialize, Serialize};

use crate::store::StoreConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub control: Option<ControlNodeConfig>,
    pub store: Option<StoreNodeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlNodeConfig {
    stores: Vec<StoreConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreNodeConfig {
    pub config: StoreConfig,
}
