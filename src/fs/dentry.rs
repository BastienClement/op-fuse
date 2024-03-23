use fuser::FileType;

use super::Inode;

/// A Directory entry. Used when reading directories.
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub inode: Inode,
    pub name: String,
    pub file_type: FileType,
}
