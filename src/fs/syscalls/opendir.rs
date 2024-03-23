use super::prelude::*;

/// Implements the `opendir` syscall.
/// Opens a directory and returns a file handle to its entries.
/// The complete entry list is stored with the file handle.
pub fn opendir(fs: &mut Fs, ino: Inode) -> Result<FileHandle> {
    let entries: Result<Vec<DirEntry>> = match &*fs.node_get(ino) {
        Node::Dummy => return Err(ENOENT),
        Node::Root(node) => Ok(from_entries_ref(node.entries(fs))),
        Node::Account(node) => Ok(from_entries_ref(node.entries(fs))),
        Node::Vault(node) => try_from_entries(node.entries(fs)),
        Node::Secret(node) => try_from_entries(node.entries(fs)),
        Node::Field(_) | Node::Link(_) => return Err(ENOTDIR),
    };

    entries.map(|entries| fs.slab_alloc(DirectoryEntries(ino, entries)))
}

fn from_entries_ref<'a, I>(entries: I) -> Vec<DirEntry>
where
    I: Iterator<Item = &'a DirEntry>,
{
    entries.map(DirEntry::clone).collect()
}

fn try_from_entries<I, E>(res: std::result::Result<I, E>) -> Result<Vec<DirEntry>>
where
    I: Iterator<Item = DirEntry>,
{
    res.map_err(|_| EIO).map(Iterator::collect)
}

/// Implements the `readdir` syscall.
/// Reads directory entries from a file handle.
pub fn readdir(
    fs: &mut Fs,
    ino: Inode,
    fh: FileHandle,
    offset: i64,
) -> Result<impl Iterator<Item = (i64, &DirEntry)>> {
    let list = match fs.slab_get(fh) {
        Some(DirectoryEntries(dl_ino, list)) if *dl_ino == ino => list,
        _ => return Err(EBADF),
    };

    let offset = usize::try_from(offset).expect("offset should be convertible to usize");

    Ok(list[offset..].iter().enumerate().map(move |(i, entry)| {
        let offset = i64::try_from(offset + i + 1).expect("offset should fit i64");
        (offset, entry)
    }))
}

/// Implements the `releasedir` syscall.
/// Closes a directory file handle and frees its resources.
pub fn releasedir(fs: &mut Fs, ino: Inode, fh: FileHandle) -> Result {
    match fs.slab_get(fh) {
        Some(DirectoryEntries(dl_ino, _)) if *dl_ino == ino => {
            fs.slab_free(fh);
            Ok(())
        }
        _ => Err(EBADF),
    }
}
