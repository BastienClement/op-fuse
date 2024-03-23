use super::prelude::*;

/// Implements the `getattr` syscall.
/// Reads the attributes of the file with the given inode.
pub fn getattr(fs: &Fs, ino: Inode) -> Result<FileAttr> {
    fs.node_get(ino).attr(fs).ok_or(ENOENT)
}
