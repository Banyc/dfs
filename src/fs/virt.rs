use std::{
    collections::HashMap,
    io::{self, Write},
    num::NonZeroUsize,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use tokio::task::spawn_blocking;

use super::block::BlockId;

#[derive(Debug, Clone)]
pub struct OpenFileTable {
    map: HashMap<PathSplit, OpenFileAttribute>,
}
impl OpenFileTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn open(
        &mut self,
        path: PathSplit,
        write: bool,
        now: Instant,
    ) -> Result<(), OpenExclusionError> {
        if self
            .map
            .get(&path)
            .is_some_and(|attr| attr.write() || write)
        {
            return Err(OpenExclusionError { path });
        }
        let Some(attr) = self.map.get_mut(&path) else {
            self.map.insert(path, OpenFileAttribute::new(write, now));
            return Ok(());
        };
        attr.read();
        Ok(())
    }
    pub fn lease(&mut self, path: &PathSplit, now: Instant) -> Result<(), LeaseNotFoundError> {
        let Some(attr) = self.map.get_mut(path) else {
            return Err(LeaseNotFoundError);
        };
        attr.lease(now);
        Ok(())
    }
    pub fn close(&mut self, path: &PathSplit) {
        let Some(attr) = self.map.get_mut(path) else {
            return;
        };
        attr.close();
        if attr.is_free() {
            self.map.remove(path).unwrap();
        }
    }
    pub fn clear_timeout(&mut self, ttl: Duration, now: Instant) {
        let mut timed_out = vec![];
        for (path, attr) in &self.map {
            if attr.is_timeout(ttl, now) {
                timed_out.push(path.clone());
            }
        }
        for path in timed_out {
            self.map.remove(&path);
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenExclusionError {
    pub path: PathSplit,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseNotFoundError;
impl Default for OpenFileTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct OpenFileAttribute {
    write: bool,
    last_lease: Instant,
    holders: usize,
}
impl OpenFileAttribute {
    pub fn new(write: bool, now: Instant) -> Self {
        Self {
            write,
            last_lease: now,
            holders: 1,
        }
    }
    pub fn read(&mut self) {
        if self.write {
            return;
        }
        self.holders += 1;
    }
    pub fn write(&self) -> bool {
        self.write
    }
    pub fn lease(&mut self, now: Instant) {
        self.last_lease = now;
    }
    pub fn close(&mut self) {
        self.holders = self.holders.saturating_sub(1);
    }
    pub fn is_free(&self) -> bool {
        self.holders == 0
    }
    pub fn is_timeout(&self, ttl: Duration, now: Instant) -> bool {
        let unrefreshed_for = now.duration_since(self.last_lease);
        ttl < unrefreshed_for
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsNode {
    attr: FsNodeAttribute,
    body: FsNodeBody,
}
impl FsNode {
    pub fn new(attr: FsNodeAttribute, body: FsNodeBody) -> Self {
        Self { attr, body }
    }
    pub fn body(&self) -> &FsNodeBody {
        &self.body
    }
    pub fn body_mut(&mut self) -> &mut FsNodeBody {
        &mut self.body
    }
    pub fn list(
        &self,
        path: Option<PathCursor>,
        mut visit: impl FnMut(&FsNode),
    ) -> Result<(), FsNodeQueryError> {
        let Some(path) = path else {
            match &self.body {
                FsNodeBody::Directory(directory) => {
                    directory.nodes().values().for_each(visit);
                }
                FsNodeBody::File(_) => {
                    visit(self);
                }
            }
            return Ok(());
        };
        match &self.body {
            FsNodeBody::Directory(directory) => {
                let Some(node) = directory.nodes().get(path.curr()) else {
                    return Err(FsNodeQueryError::FileNotExist(FileNotExist { path }));
                };
                node.list(path.next(), visit)
            }
            FsNodeBody::File(_) => Err(FsNodeQueryError::DirectoryNotExist(DirectoryNotExist {
                path,
            })),
        }
    }
    pub fn get(&self, path: Option<PathCursor>) -> Result<&FsNode, FsNodeQueryError> {
        let Some(path) = path else {
            return Ok(self);
        };
        match &self.body {
            FsNodeBody::Directory(directory) => {
                let Some(node) = directory.nodes().get(path.curr()) else {
                    return Err(FsNodeQueryError::FileNotExist(FileNotExist { path }));
                };
                node.get(path.next())
            }
            FsNodeBody::File(_) => Err(FsNodeQueryError::DirectoryNotExist(DirectoryNotExist {
                path,
            })),
        }
    }
    pub fn get_mut(&mut self, path: Option<PathCursor>) -> Result<&mut FsNode, FsNodeQueryError> {
        let Some(path) = path else {
            return Ok(self);
        };
        match &mut self.body {
            FsNodeBody::Directory(directory) => {
                let Some(node) = directory.nodes_mut().get_mut(path.curr()) else {
                    return Err(FsNodeQueryError::FileNotExist(FileNotExist { path }));
                };
                node.get_mut(path.next())
            }
            FsNodeBody::File(_) => Err(FsNodeQueryError::DirectoryNotExist(DirectoryNotExist {
                path,
            })),
        }
    }
    pub fn create_node(
        &mut self,
        path: PathCursor,
        new_node: impl FnOnce() -> FsNode,
    ) -> Result<(), FsNodeCreateFileError> {
        let directory = match &mut self.body {
            FsNodeBody::Directory(directory) => directory,
            FsNodeBody::File(_) => {
                return Err(FsNodeCreateFileError::DirectoryNotExist(
                    DirectoryNotExist { path },
                ))
            }
        };
        let child = path.next();
        match child {
            Some(child) => {
                let Some(node) = directory.nodes_mut().get_mut(path.curr()) else {
                    return Err(FsNodeCreateFileError::DirectoryNotExist(
                        DirectoryNotExist { path },
                    ));
                };
                node.create_node(child, new_node)
            }
            None => {
                let file_name = path.curr();
                if directory.nodes_mut().get(file_name).is_some() {
                    return Err(FsNodeCreateFileError::FileExist(FileExist { path }));
                }
                directory
                    .nodes_mut()
                    .insert(path.curr().clone(), new_node());
                Ok(())
            }
        }
    }
}
#[derive(Debug, Clone)]
pub enum FsNodeQueryError {
    FileNotExist(FileNotExist),
    DirectoryNotExist(DirectoryNotExist),
}
#[derive(Debug, Clone)]
pub enum FsNodeCreateFileError {
    FileExist(FileExist),
    DirectoryNotExist(DirectoryNotExist),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsNodeAttribute {
    // name: Arc<str>,
}
impl FsNodeAttribute {
    pub fn new() -> Self {
        Self {}
    }
}
impl Default for FsNodeAttribute {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FsNodeBody {
    Directory(Directory),
    File(File),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    attr: DirectoryAttribute,
    nodes: HashMap<Arc<str>, FsNode>,
}
impl Directory {
    pub fn new(attr: DirectoryAttribute) -> Self {
        Self {
            attr,
            nodes: HashMap::new(),
        }
    }
    pub fn insert(&mut self, key: Arc<str>, node: FsNode) -> Result<(), DirectoryInsertError> {
        if self.nodes.contains_key(&key) {
            return Err(DirectoryInsertError { node });
        }
        self.nodes.insert(key, node);
        Ok(())
    }
    pub fn nodes(&self) -> &HashMap<Arc<str>, FsNode> {
        &self.nodes
    }
    pub fn nodes_mut(&mut self) -> &mut HashMap<Arc<str>, FsNode> {
        &mut self.nodes
    }
}
#[derive(Debug, Clone)]
pub struct DirectoryInsertError {
    pub node: FsNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryAttribute {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    attr: FileAttribute,
    blocks: Vec<FileBlock>,
}
impl File {
    pub fn new(attr: FileAttribute) -> Self {
        Self {
            attr,
            blocks: vec![],
        }
    }
    pub fn attr(&self) -> &FileAttribute {
        &self.attr
    }
    pub fn blocks_mut(&mut self) -> &mut Vec<FileBlock> {
        &mut self.blocks
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttribute {
    replication: NonZeroUsize,
}
impl FileAttribute {
    pub fn new(replication: NonZeroUsize) -> Self {
        Self { replication }
    }
    pub fn replication(&self) -> NonZeroUsize {
        self.replication
    }
    pub fn set_replication(&mut self, replication: NonZeroUsize) {
        self.replication = replication;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBlock {
    off_range: (u64, u64),
    id: BlockId,
}
impl FileBlock {
    pub fn new(off_range: (u64, u64), id: BlockId) -> Self {
        Self { off_range, id }
    }
    pub fn off_range(&self) -> (u64, u64) {
        self.off_range
    }
    pub fn id(&self) -> &BlockId {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub struct PathCursor {
    path_split: PathSplit,
    curr: usize,
}
impl PathCursor {
    pub fn new(path_split: PathSplit) -> Option<Self> {
        if path_split.segs().is_empty() {
            return None;
        }
        Some(Self {
            path_split,
            curr: 0,
        })
    }
    pub fn curr(&self) -> &Arc<str> {
        &self.path_split.segs()[self.curr]
    }
    pub fn next(&self) -> Option<Self> {
        if self.curr + 1 == self.path_split.segs().len() {
            return None;
        }
        Some(Self {
            path_split: self.path_split.clone(),
            curr: self.curr + 1,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathSplit {
    segs: Arc<[Arc<str>]>,
}
impl PathSplit {
    pub fn from_uri(path_str: &str) -> Self {
        let segs = path_str
            .trim()
            .split('/')
            .filter(|s| !s.trim().is_empty())
            .map(Arc::from)
            .collect();
        Self { segs }
    }
    pub fn segs(&self) -> &Arc<[Arc<str>]> {
        &self.segs
    }
}

#[derive(Debug, Clone)]
pub struct FileExist {
    pub path: PathCursor,
}
#[derive(Debug, Clone)]
pub struct FileNotExist {
    pub path: PathCursor,
}
#[derive(Debug, Clone)]
pub struct DirectoryNotExist {
    pub path: PathCursor,
}

pub async fn atomic_persist(path: impl AsRef<Path>, buf: &[u8]) -> io::Result<()> {
    let path = path.as_ref().to_path_buf();
    let buf = unsafe { Arc::from_raw(buf) };
    spawn_blocking({
        let buf = buf.clone();
        move || -> io::Result<()> {
            let mut file = NamedTempFile::new()?;
            file.write_all(&buf)?;
            file.flush()?;
            file.as_file().sync_all()?;
            file.persist(path)?;
            Ok(())
        }
    })
    .await??;
    drop(buf);
    Ok(())
}
