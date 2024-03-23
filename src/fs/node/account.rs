use std::{cell::OnceCell, time::SystemTime};

use fuser::{FileAttr, FileType};

use crate::{
    fs::{dentry::DirEntry, Fs, Inode},
    onepassword::id,
};

use super::Node;

/// An account node.
pub struct Account {
    /// The inode number of the node.
    ino: Inode,

    /// The ID of the account.
    id: id::Account,

    /// The cached file attributes of the node.
    attr: OnceCell<FileAttr>,

    /// The cached directory entries of the node.
    entries: OnceCell<Vec<DirEntry>>,
}

impl Account {
    /// Creates a new account node.
    pub fn new(ino: Inode, id: id::Account) -> Account {
        Self {
            ino,
            id,
            attr: OnceCell::new(),
            entries: OnceCell::new(),
        }
    }

    /// Returns the file attributes of the node.
    pub fn attr(&self, fs: &Fs) -> FileAttr {
        *self.attr.get_or_init(|| make_attr(self, fs))
    }

    /// Returns the directory entries of the node.
    pub fn entries(&self, fs: &Fs) -> impl Iterator<Item = &DirEntry> {
        self.entries.get_or_init(|| make_entries(self, fs)).iter()
    }
}

/// Creates the file attributes of an account node.
fn make_attr(node: &Account, fs: &Fs) -> FileAttr {
    let now = SystemTime::now();
    FileAttr {
        ino: node.ino,
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

/// Creates the directory entries of an account node.
/// Each entry represents a vault in the account.
fn make_entries(node: &Account, fs: &Fs) -> Vec<DirEntry> {
    let mut entries = Vec::new();
    for vault in fs.config.accounts[node.id.account()].vaults.keys() {
        let entry = DirEntry {
            inode: fs
                .node_alloc(|ino| Node::new_vault(ino, id::Vault::new(&node.id, vault)))
                .persist(),
            name: vault.to_string(),
            file_type: FileType::Directory,
        };

        entries.push(entry);
    }

    entries
}
