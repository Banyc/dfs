use serde::{Deserialize, Serialize};

use crate::fs::block::{BlockId, BlockReport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlReq {
    OpenReq(OpenReq),
    OpenLeaseReq(OpenLeaseReq),
    CloseReq(CloseReq),
    AllocBlockReq(AllocBlockReq),
    BlockReportReq(BlockReportReq),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenReq {
    pub write: bool,
    pub path: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenResp {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenLeaseReq {
    pub path: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenLeaseResp {
    pub permitted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseReq {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFile {
    pub path: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDirectory {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocBlockReq {
    pub path: String,
    pub off_range: (u64, u64),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocBlockResp {
    Ok(AllocBlockRespOk),
    Rejected,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocBlockRespOk {
    pub block: BlockId,
    pub store_addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockReportReq {
    report: BlockReport,
}
