use super::dentry::DirEntry;

/// A slab item.
#[derive(Debug)]
pub enum Item {
    /// Stores directories entries during readdir.
    DirectoryEntries(u64, Vec<DirEntry>),
}
