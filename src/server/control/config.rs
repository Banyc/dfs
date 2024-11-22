use serde::{Deserialize, Serialize};

use crate::store::StoreConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlNodeConfig {
    stores: Vec<StoreConfig>,
}
