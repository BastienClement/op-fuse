use std::{cell::OnceCell, time::SystemTime};

use fuser::{FileAttr, FileType};

use crate::{
    fs::{dentry::DirEntry, Fs},
    onepassword::id,
};

use super::Node;

/// The root node.
pub struct Root {
    /// The cached file attributes of the node.
    attr: FileAttr,

    /// The cached directory entries of the node.
    entries: OnceCell<Vec<DirEntry>>,
}

impl Root {
    /// Creates a new root node.
    pub fn new(fs: &Fs) -> Self {
        Root {
            attr: make_attr(fs),
            entries: OnceCell::new(),
        }
    }

    /// Returns the file attributes of the node.
    pub fn attr(&self) -> FileAttr {
        self.attr
    }

    /// Returns the directory entries of the node.
    pub fn entries(&self, fs: &Fs) -> impl Iterator<Item = &DirEntry> {
        self.entries.get_or_init(|| make_entries(fs)).iter()
    }
}

/// Creates the file attributes of the root node.
fn make_attr(fs: &Fs) -> FileAttr {
    let now = SystemTime::now();
    FileAttr {
        ino: fuser::FUSE_ROOT_ID,
        size: 512,
        blocks: 0,
        atime: now,
        mtime: now,
        ctime: now,
        crtime: now,
        kind: FileType::Directory,
        perm: fs.config.dir_mode,
        nlink: 1,
        uid: fs.config.uid,
        gid: fs.config.gid,
        rdev: 0,
        flags: 0,
        blksize: 512,
    }
}

/// Creates the directory entries of the root node.
/// Each account is represented as a directory.
fn make_entries(fs: &Fs) -> Vec<DirEntry> {
    let mut entries = Vec::new();
    for account in fs.config.accounts.keys() {
        let entry = DirEntry {
            inode: fs
                .node_alloc(|ino| Node::new_account(ino, id::Account::new(account)))
                .persist(),
            name: account.to_string(),
            file_type: FileType::Directory,
        };
        entries.push(entry);
    }
    entries
}
