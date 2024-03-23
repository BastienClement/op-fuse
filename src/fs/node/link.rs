use fuser::{FileAttr, FileType};

use crate::fs::Inode;

/// A symbolic link node.
pub struct Link {
    /// The target of the link.
    target: String,

    /// The attributes of the link.
    attr: FileAttr,
}

impl Link {
    /// Creates a new link node.
    pub fn new(ino: Inode, target: &str, attr: &FileAttr) -> Link {
        Self {
            target: target.to_string(),
            attr: FileAttr {
                ino,
                size: target.len() as u64,
                kind: FileType::Symlink,
                ..*attr
            },
        }
    }

    /// Returns the file attributes of the node.
    pub fn attr(&self) -> FileAttr {
        self.attr
    }

    /// Returns the target of the link.
    pub fn target(&self) -> &str {
        &self.target
    }
}
