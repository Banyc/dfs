use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

pub type StoreId = Arc<str>;

#[derive(Debug, Clone)]
pub struct StoreStatusesMap {
    map: HashMap<StoreId, StoreStatus>,
}
impl StoreStatusesMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn insert(&mut self, store: StoreId, config: StoreConfig) {
        assert!(!self.map.contains_key(&store));
        self.map.insert(store, StoreStatus::new(config));
    }
    pub fn get_mut(&mut self, store: &StoreId) -> Option<&mut StoreStatus> {
        self.map.get_mut(store)
    }
}
impl Default for StoreStatusesMap {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct StoreStatus {
    config: StoreConfig,
    last_heartbeat: Option<Instant>,
}
impl StoreStatus {
    pub fn new(config: StoreConfig) -> Self {
        Self {
            config,
            last_heartbeat: None,
        }
    }
    pub fn config(&self) -> &StoreConfig {
        &self.config
    }
    pub fn beat(&mut self, now: Instant) {
        self.last_heartbeat = Some(now);
    }
    pub fn is_alive(&self, ttl: Duration, now: Instant) -> bool {
        let Some(last_heartbeat) = self.last_heartbeat else {
            return false;
        };
        let stop_beat_for = now.duration_since(last_heartbeat);
        stop_beat_for <= ttl
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    addr: SocketAddr,
}
impl StoreConfig {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}
