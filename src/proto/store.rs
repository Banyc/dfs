use serde::{Deserialize, Serialize};

use crate::fs::block::{BlockId, BlockReport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoreProto {
    OpenBlockReq(OpenBlockReq),
    OpenBlockResp(OpenBlockResp),
    ReplicateBlockReq(ReplicateBlockReq),
    ReplicateBlockResp(ReplicateBlockResp),
    RemoveBlockReq(RemoveBlockReq),
    RemoveBlockResp(RemoveBlockResp),
    HeartbeatReq(HeartbeatReq),
    HeartbeatResp(HeartbeatResp),
    FullBlockReportReq(FullBlockReportReq),
    FullBlockReportResp(FullBlockReportResp),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenBlockReq {
    pub block: BlockId,
    pub write: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenBlockResp {
    pub permitted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicateBlockReq {
    pub block: BlockId,
    pub store_addr: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicateBlockResp {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveBlockReq {
    pub block: BlockId,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveBlockResp {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatReq {}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResp {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullBlockReportReq {}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullBlockReportResp {
    pub report: BlockReport,
}
