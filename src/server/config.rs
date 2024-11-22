use serde::{Deserialize, Serialize};

use super::{control::config::ControlNodeConfig, store::config::StoreNodeConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub control: Option<ControlNodeConfig>,
    pub store: Option<StoreNodeConfig>,
}
