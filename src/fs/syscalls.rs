mod getattr;
mod lookup;
mod opendir;
mod read;

pub use getattr::getattr;
pub use lookup::lookup;
pub use opendir::{opendir, readdir, releasedir};
pub use read::{read, read_link};

pub type Result<T = ()> = std::result::Result<T, libc::c_int>;

mod prelude {
    #[allow(clippy::wildcard_imports)]
    pub use libc::*;

    pub use fuser::FileAttr;
    pub use std::ffi::OsStr;

    pub use crate::fs::dentry::DirEntry;
    pub use crate::fs::node::Node;
    pub use crate::fs::slab::Item::*;
    pub use crate::fs::FileHandle;
    pub use crate::fs::Fs;
    pub use crate::fs::Inode;

    pub use super::Result;
}
