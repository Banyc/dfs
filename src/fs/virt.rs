use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

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
                    return Err(FsNodeQueryError::PathNotExist(PathNotExist { path }));
                };
                node.list(path.next(), visit)
            }
            FsNodeBody::File(_) => Err(FsNodeQueryError::PathNotDirectory(PathNotDirectory {
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
                    return Err(FsNodeQueryError::PathNotExist(PathNotExist { path }));
                };
                node.get(path.next())
            }
            FsNodeBody::File(_) => Err(FsNodeQueryError::PathNotDirectory(PathNotDirectory {
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
                    return Err(FsNodeQueryError::PathNotExist(PathNotExist { path }));
                };
                node.get_mut(path.next())
            }
            FsNodeBody::File(_) => Err(FsNodeQueryError::PathNotDirectory(PathNotDirectory {
                path,
            })),
        }
    }
}
#[derive(Debug, Clone)]
pub enum FsNodeQueryError {
    PathNotExist(PathNotExist),
    PathNotDirectory(PathNotDirectory),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsNodeAttribute {
    name: Arc<str>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttribute {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBlock {
    off_range: (u64, u64),
    replication: Vec<PhysBlockLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysBlockLocation {
    host_name: Arc<str>,
    path: Arc<str>,
}
impl PhysBlockLocation {
    pub fn new(host_name: Arc<str>, path: Arc<str>) -> Self {
        Self { host_name, path }
    }
    pub fn host_name(&self) -> &Arc<str> {
        &self.host_name
    }
    pub fn path(&self) -> &Arc<str> {
        &self.path
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

#[derive(Debug, Clone)]
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
pub struct PathNotExist {
    pub path: PathCursor,
}
#[derive(Debug, Clone)]
pub struct PathNotDirectory {
    pub path: PathCursor,
}
