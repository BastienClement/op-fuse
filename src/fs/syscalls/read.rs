use crate::fs::node::field::Field;

use super::prelude::*;

/// Implements the `read` syscall.
/// Reads data from a file.
pub fn read(fs: &Fs, ino: Inode, offset: i64, size: u32) -> Result<Vec<u8>> {
    match &*fs.node_get(ino) {
        Node::Dummy => Err(ENOENT),
        Node::Field(node) => read_field(node, offset, size),
        Node::Link(_) => Err(EIO), // Should call `readlink` instead
        _ => Err(EISDIR),
    }
}

fn read_field(field: &Field, offset: i64, size: u32) -> Result<Vec<u8>> {
    Ok(field.read(
        usize::try_from(offset).map_err(|_| EINVAL)?,
        usize::try_from(size).map_err(|_| EINVAL)?,
    ))
}

/// Implements the `readlink` syscall.
/// Reads the target of a symbolic link.
pub fn read_link(fs: &Fs, ino: Inode) -> Result<String> {
    match &*fs.node_get(ino) {
        Node::Dummy => Err(ENOENT),
        Node::Link(node) => Ok(node.target().to_string()),
        _ => Err(EIO),
    }
}
