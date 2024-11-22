use std::{
    num::NonZeroUsize,
    time::{Duration, Instant},
};

use crate::{
    fs::{
        block::ReplicatedBlocksMap,
        virt::{
            File, FileAttribute, FileBlock, FsNode, FsNodeAttribute, FsNodeBody,
            FsNodeCreateFileError, OpenFileTable, PathCursor, PathSplit,
        },
    },
    proto::control::{AllocBlockResp, AllocBlockRespOk, ControlReq, OpenLeaseResp, OpenResp},
    store::StoreStatusesMap,
};

const OPEN_LEASE_TTL: Duration = Duration::from_secs(60);
const REPLICATION: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(3) };

#[derive(Debug, Clone)]
pub struct Handler {
    virt_fs: FsNode,
    open_table: OpenFileTable,
    store_statuses: StoreStatusesMap,
    replicated_blocks: ReplicatedBlocksMap,
}
impl Handler {
    pub fn new(
        virt_fs: FsNode,
        open_table: OpenFileTable,
        store_statuses: StoreStatusesMap,
        replicated_blocks: ReplicatedBlocksMap,
    ) -> Self {
        Self {
            virt_fs,
            open_table,
            store_statuses,
            replicated_blocks,
        }
    }
    pub fn handle_timer(&mut self) {
        let now = Instant::now();
        self.open_table.clear_timeout(OPEN_LEASE_TTL, now);
        todo!()
    }
    pub fn handle_req(&mut self, msg: ControlReq) -> Resp {
        let now = Instant::now();
        match msg {
            ControlReq::OpenReq(open_req) => {
                let path = PathSplit::from_uri(&open_req.path);
                let path_cursor = PathCursor::new(path.clone());
                if open_req.write {
                    let Some(path_cursor) = path_cursor else {
                        return Resp::OpenResp(OpenResp {});
                    };
                    let res = self.virt_fs.create_node(path_cursor, || {
                        FsNode::new(
                            FsNodeAttribute::new(),
                            FsNodeBody::File(File::new(FileAttribute::new(REPLICATION))),
                        )
                    });
                    match res {
                        Ok(_) => (),
                        Err(e) => match e {
                            FsNodeCreateFileError::FileExist(_) => (),
                            FsNodeCreateFileError::DirectoryNotExist(_) => {
                                return Resp::OpenResp(OpenResp {});
                            }
                        },
                    }
                } else {
                    let Ok(node) = self.virt_fs.get(path_cursor) else {
                        return Resp::OpenResp(OpenResp {});
                    };
                    let FsNodeBody::File(_) = node.body() else {
                        return Resp::OpenResp(OpenResp {});
                    };
                }
                let res = self.open_table.open(path, open_req.write, now);
                match res {
                    Ok(_) => Resp::None,
                    Err(_) => Resp::OpenResp(OpenResp {}),
                }
            }
            ControlReq::OpenLeaseReq(open_lease_req) => {
                let path = PathSplit::from_uri(&open_lease_req.path);
                let res = self.open_table.lease(&path, now);
                match res {
                    Ok(_) => Resp::OpenLeaseResp(OpenLeaseResp { permitted: true }),
                    Err(_) => Resp::OpenLeaseResp(OpenLeaseResp { permitted: false }),
                }
            }
            ControlReq::CloseReq(close_req) => {
                let path = PathSplit::from_uri(&close_req.path);
                self.open_table.close(&path);
                Resp::None
            }
            ControlReq::AllocBlockReq(alloc_block_req) => {
                let path = PathSplit::from_uri(&alloc_block_req.path);
                let path = PathCursor::new(path);
                let res = self.virt_fs.get_mut(path);
                let node = match res {
                    Ok(fs_node) => fs_node,
                    Err(_) => return Resp::AllocBlockResp(AllocBlockResp::Rejected),
                };
                let file = match node.body_mut() {
                    FsNodeBody::Directory(_) => {
                        return Resp::AllocBlockResp(AllocBlockResp::Rejected);
                    }
                    FsNodeBody::File(file) => file,
                };
                let off_range = alloc_block_req.off_range;
                if let Some(last) = file.blocks_mut().last() {
                    let (_, last) = last.off_range();
                    if off_range.0 != last {
                        return Resp::AllocBlockResp(AllocBlockResp::Rejected);
                    }
                }
                let id: std::sync::Arc<str> = todo!();
                let block = FileBlock::new(off_range, id.clone());
                file.blocks_mut().push(block);
                Resp::AllocBlockResp(AllocBlockResp::Ok(AllocBlockRespOk {
                    block: id,
                    store_addr: todo!(),
                }))
            }
            ControlReq::BlockReportReq(block_report_req) => todo!(),
        }
    }
}

pub enum Resp {
    None,
    OpenResp(OpenResp),
    OpenLeaseResp(OpenLeaseResp),
    AllocBlockResp(AllocBlockResp),
}
