use std::borrow::Borrow;

use super::prelude::*;

/// Implements the `lookup` syscall.
/// Looks up a directory entry by name and returns its inode.
pub fn lookup(fs: &Fs, parent: Inode, name: &OsStr) -> Result<Inode> {
    let Some(name) = name.to_str() else {
        return Err(ENOENT);
    };

    match &*fs.node_get(parent) {
        Node::Dummy => Err(ENOENT),
        Node::Root(node) => scan_entries(name, node.entries(fs)),
        Node::Account(node) => scan_entries(name, node.entries(fs)),
        Node::Vault(node) => try_scan_entries(name, node.entries(fs)),
        Node::Secret(node) => try_scan_entries(name, node.entries(fs)),
        Node::Field(_) | Node::Link(_) => Err(ENOTDIR),
    }
}

/// Scan an iterator of directory entries for a given name and return its
/// inode, or ENOENT if not found.
/// It avoids implementing a lookup mechanism if listing the entries is cheap.
fn scan_entries<I, E>(name: &str, mut it: I) -> Result<Inode>
where
    I: Iterator<Item = E>,
    E: Borrow<DirEntry>,
{
    it.find(|e| e.borrow().name == name)
        .map(|e| e.borrow().inode)
        .ok_or(ENOENT)
}

fn try_scan_entries<I, E, Err>(name: &str, res: std::result::Result<I, Err>) -> Result<Inode>
where
    I: Iterator<Item = E>,
    E: Borrow<DirEntry>,
{
    res.map_err(|_| EIO).and_then(|it| scan_entries(name, it))
}
