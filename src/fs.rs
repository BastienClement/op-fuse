use std::{ffi::OsStr, rc::Rc};

#[allow(clippy::wildcard_imports)]
use fuser::*;
use libc::c_int;

use crate::{fs::node::Node, Config, OnePassword};
use ::slab::Slab;

mod dentry;
mod node;
mod slab;
mod syscalls;

/// A pointer to a filesystem node
pub type Inode = u64;

/// A pointer to a slab item
pub type FileHandle = u64;

/// A dummy TTL of 0 seconds
///
/// Currently used for all cachable replies. That way the kernel will always
/// revalidate the cache. It will become handy when we implement access control.
pub const TTL_ZERO: std::time::Duration = std::time::Duration::from_secs(0);

/// The 1Password-Fuse filesystem
pub struct Fs {
    config: Config,
    op: OnePassword,
    nodes: node::Set,
    slab: Slab<slab::Item>,
}

impl Fs {
    /// Creates a new filesystem from the given config and 1Password client
    pub fn new(config: &Config, op: OnePassword) -> Fs {
        let fs = Fs {
            config: config.clone(),
            op,
            nodes: node::Set::new(),
            slab: Slab::new(),
        };

        assert_eq!(0, fs.node_alloc(|_| Node::new_dummy()).persist());
        assert_eq!(
            FUSE_ROOT_ID,
            fs.node_alloc(|_| Node::new_root(&fs)).persist()
        );

        fs
    }

    /**
     * Node management
     */

    /// Allocates a new node
    fn node_alloc<F>(&self, node: F) -> node::Handler
    where
        F: FnOnce(Inode) -> node::Node,
    {
        self.nodes.alloc(node)
    }

    /// Gets a node by its inode
    fn node_get(&self, ino: Inode) -> Rc<node::Node> {
        self.nodes.get(ino)
    }

    /**
     * Slab management
     */

    /// Allocates a new slab item
    fn slab_alloc(&mut self, item: slab::Item) -> FileHandle {
        self.slab.insert(item) as FileHandle
    }

    /// Gets a slab item by its file handle
    fn slab_get(&self, fh: FileHandle) -> Option<&slab::Item> {
        self.slab.get(u64_to_usize(fh))
    }

    /// Gets a mutable slab item by its file handle
    #[allow(dead_code)]
    fn slab_get_mut(&mut self, fh: FileHandle) -> Option<&mut slab::Item> {
        self.slab.get_mut(u64_to_usize(fh))
    }

    /// Frees a slab item by its file handle
    #[allow(dead_code)]
    fn slab_free(&mut self, fh: FileHandle) {
        self.slab.remove(u64_to_usize(fh));
    }
}

/// Converts a 64-bit unsigned integer to a usize
fn u64_to_usize(x: u64) -> usize {
    usize::try_from(x).expect("pointers should be 64 bits")
}

impl fuser::Filesystem for Fs {
    fn getattr(&mut self, _req: &Request, ino: Inode, reply: ReplyAttr) {
        match syscalls::getattr(self, ino) {
            Ok(attr) => reply.attr(&TTL_ZERO, &attr),
            Err(errno) => reply.error(trace_err(errno)),
        }
    }

    fn opendir(&mut self, _req: &Request, ino: Inode, _flags: i32, reply: ReplyOpen) {
        match syscalls::opendir(self, ino) {
            Ok(fh) => reply.opened(fh, 0),
            Err(errno) => reply.error(trace_err(errno)),
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: Inode,
        fh: FileHandle,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        match syscalls::readdir(self, ino, fh, offset) {
            Ok(entries) => {
                for (offset, entry) in entries {
                    if reply.add(entry.inode, offset, entry.file_type, &entry.name) {
                        break;
                    }
                }
                reply.ok();
            }
            Err(errno) => reply.error(trace_err(errno)),
        }
    }

    fn releasedir(
        &mut self,
        _req: &Request,
        ino: Inode,
        fh: FileHandle,
        _flags: i32,
        reply: ReplyEmpty,
    ) {
        match syscalls::releasedir(self, ino, fh) {
            Ok(()) => reply.ok(),
            Err(errno) => reply.error(trace_err(errno)),
        }
    }

    fn lookup(&mut self, _req: &Request, parent: Inode, name: &OsStr, reply: ReplyEntry) {
        match syscalls::lookup(self, parent, name).and_then(|ino| syscalls::getattr(self, ino)) {
            Ok(attr) => reply.entry(&TTL_ZERO, &attr, 1),
            Err(errno) => reply.error(trace_err(errno)),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: Inode,
        _fh: Inode,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        match syscalls::read(self, ino, offset, size) {
            Ok(data) => reply.data(&data),
            Err(errno) => reply.error(trace_err(errno)),
        }
    }

    fn flush(
        &mut self,
        _req: &Request,
        _ino: Inode,
        _fh: u64,
        _lock_owner: u64,
        reply: ReplyEmpty,
    ) {
        reply.ok();
    }

    fn release(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        // Not used as we don't keep file handles
        reply.ok();
    }

    fn readlink(&mut self, _req: &Request, ino: Inode, reply: ReplyData) {
        match syscalls::read_link(self, ino) {
            Ok(target) => reply.data(target.as_bytes()),
            Err(errno) => reply.error(trace_err(errno)),
        }
    }
}

/// Logs an error and returns it
fn trace_err(err: c_int) -> c_int {
    if let Some(name) = err_name(err) {
        warn!(err = %name);
    } else {
        warn!(err = %err);
    }
    err
}

/// Returns the name of a libc error code, if known
fn err_name(err: c_int) -> Option<&'static str> {
    use libc::{EINVAL, EIO, EISDIR, ENOENT};
    Some(match err {
        ENOENT => "ENOENT",
        EIO => "EIO",
        EISDIR => "EISDIR",
        EINVAL => "EINVAL",
        _ => return None,
    })
}
