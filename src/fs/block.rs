use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::store::StoreId;

use super::virt::PathSplit;

pub type BlockId = Arc<str>;

#[derive(Debug, Clone)]
pub struct ReplicatedBlocksMap {
    map: HashMap<BlockId, ReplicatedBlock>,
}
impl ReplicatedBlocksMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn insert(&mut self, id: BlockId, block: ReplicatedBlock) {
        assert!(!self.map.contains_key(&id));
        self.map.insert(id, block);
    }
    pub fn remove(&mut self, id: &BlockId) {
        assert!(self.map.remove(id).is_some());
    }
    pub fn push_store(
        &mut self,
        store: StoreId,
        block: ReportedBlock,
    ) -> Result<(), CorruptedBlockError> {
        let Some(b) = self.map.get_mut(block.id()) else {
            return Err(CorruptedBlockError { store });
        };
        b.push(store, block.body())?;
        Ok(())
    }
    pub fn stores(&self, block: &BlockId) -> &[StoreId] {
        self.map
            .get(block)
            .map(|x| x.stores())
            .unwrap_or_else(|| &[])
    }
}
impl Default for ReplicatedBlocksMap {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ReplicatedBlock {
    body: BlockBody,
    stores: Vec<StoreId>,
    virt_path: PathSplit,
}
impl ReplicatedBlock {
    pub fn new(body: BlockBody, virt_path: PathSplit) -> Self {
        Self {
            body,
            stores: vec![],
            virt_path,
        }
    }
    pub fn body(&self) -> &BlockBody {
        &self.body
    }
    pub fn stores(&self) -> &[StoreId] {
        &self.stores
    }
    pub fn virt_path(&self) -> &PathSplit {
        &self.virt_path
    }
    pub fn push(&mut self, store: StoreId, body: &BlockBody) -> Result<(), CorruptedBlockError> {
        if self.body != *body {
            return Err(CorruptedBlockError { store });
        }
        self.stores.push(store);
        Ok(())
    }
}
pub struct CorruptedBlockError {
    pub store: StoreId,
}

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
    blocks: Vec<ReportedBlock>,
}
impl BlockList {
    pub fn new() -> Self {
        Self { blocks: vec![] }
    }
    pub fn push(&mut self, block: ReportedBlock) {
        self.blocks.push(block);
    }
}
impl Default for BlockList {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportedBlock {
    id: BlockId,
    body: BlockBody,
}
impl ReportedBlock {
    pub fn new(id: BlockId, body: BlockBody) -> Self {
        Self { id, body }
    }
    pub fn id(&self) -> &BlockId {
        &self.id
    }
    pub fn body(&self) -> &BlockBody {
        &self.body
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockBody {
    size: u32,
}
impl BlockBody {
    pub fn new(size: u32) -> Self {
        Self { size }
    }
    pub fn size(&self) -> u32 {
        self.size
    }
}
