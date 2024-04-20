use std::{collections::HashMap,
          path::PathBuf};

use serde::{Deserialize,
            Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Default)]
pub enum NodeKind {
    #[default]
    LEAF,
    BRANCH,
}

#[derive(Serialize, Deserialize, Default)]
pub struct NodeMetadata {
    pub name: String,
    pub kind: NodeKind,
    pub hash: u32,
    pub start: u64,
    pub end: u64,
}

#[derive(Serialize, Deserialize)]
pub struct MetadataContainer {
    pub magic: [u8; 16],
    pub paksize: u64,
    pub table: HashMap<PathBuf, NodeMetadata>,
}
