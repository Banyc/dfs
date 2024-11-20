use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockReport {
    ty: BlockReportType,
    body: BlockList,
}
impl BlockReport {
    pub fn new(ty: BlockReportType, body: BlockList) -> Self {
        Self { ty, body }
    }
    pub fn ty(&self) -> BlockReportType {
        self.ty
    }
    pub fn body(&self) -> &BlockList {
        &self.body
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BlockReportType {
    Add,
    Remove,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockList {
    blocks: Vec<Arc<str>>,
}
impl BlockList {
    pub fn new() -> Self {
        Self { blocks: vec![] }
    }
    pub fn push(&mut self, block: Arc<str>) {
        self.blocks.push(block);
    }
}
impl Default for BlockList {
    fn default() -> Self {
        Self::new()
    }
}
