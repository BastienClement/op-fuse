use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
    time::SystemTime,
};

use anyhow::Result;
use fuser::{FileAttr, FileType};

use crate::{
    fs::{dentry::DirEntry, Fs, Inode},
    onepassword::id,
    util::{diff, Throttle},
};

use super::{Handler, Node};

/// A vault node.
pub struct Vault {
    /// The inode number of the node.
    ino: Inode,

    /// The ID of the vault.
    id: id::Vault,

    /// The cached file attributes of the node.
    attr: OnceCell<FileAttr>,

    /// The cached secret handlers of the node.
    entries: RefCell<Throttle<HashMap<String, SecretHandler>>>,
}

/// A secret handler that contains the secret-node handler, the alias handler if
/// present.
struct SecretHandler {
    /// The secret-node handler.
    node: Handler,

    /// The alias handler if present.
    alias: Option<(String, Handler)>,
}

impl SecretHandler {
    /// Returns the alias name of the handler.
    fn alias_name(&self) -> Option<&String> {
        self.alias.as_ref().map(|(name, _)| name)
    }
}

impl Vault {
    /// Creates a new vault node.
    pub fn new(ino: Inode, id: id::Vault) -> Vault {
        Self {
            ino,
            id,
            attr: OnceCell::new(),
            entries: RefCell::new(Throttle::default()),
        }
    }

    /// Returns the file attributes of the node.
    pub fn attr(&self, fs: &Fs) -> FileAttr {
        *self.attr.get_or_init(|| make_attr(self, fs))
    }

    /// Returns the directory entries of the node.
    pub fn entries(&self, fs: &Fs) -> Result<impl Iterator<Item = DirEntry>> {
        let mut entries = self.entries.borrow_mut();
        entries.try_refresh(fs.config.cache_duration, |entries| {
            let mut secrets = fs
                .op
                .list_secrets(&self.id)?
                .into_iter()
                .map(|secret| (secret.id.clone(), secret))
                .collect::<HashMap<_, _>>();

            let (delete, update, create) = diff(entries.keys(), secrets.keys());

            for id in delete {
                entries.remove(&id);
            }

            for id in update {
                let meta = secrets.remove(&id).expect("secret should be in list");
                let title = meta.title.clone();
                let handler = entries.get_mut(&id).expect("handler should be in list");

                match handler.node.node().as_ref() {
                    Node::Secret(secret) => secret.update_metadata(meta),
                    _ => unreachable!("node should be a secret"),
                }

                // Update the alias if the title has changed
                let alias_name = make_alias_name(&title);
                if alias_name.as_ref() != handler.alias_name() {
                    match (alias_name, handler.alias.take()) {
                        // Rename: updates the existing alias by replacing the
                        // old name and keeping the existing handler.
                        (Some(new_name), Some((_, existing))) => {
                            handler.alias = Some((new_name.clone(), existing));
                        }
                        // Create: there is no existing alias.
                        (Some(alias_name), None) => {
                            let attr = handler
                                .node
                                .node()
                                .attr(fs)
                                .expect("attr should be available");
                            let alias_handler =
                                fs.node_alloc(|ino| Node::new_link(ino, &id, &attr));
                            handler.alias = Some((alias_name, alias_handler));
                        }
                        // Delete: the alias name is no longer valid.
                        // Will drop the alias as the handler is dropped.
                        (None, Some(_)) => {}
                        (None, None) => unreachable!(),
                    }
                }
            }

            for id in create {
                let meta = secrets.remove(&id).expect("secret should be in list");
                let title = meta.title.clone();

                let secret_id = id::Secret::new(&self.id, &id);
                let handler = fs.node_alloc(|ino| Node::new_secret(ino, secret_id, meta));

                // Check if we can create an alias for the secret
                let alias_handler =
                    make_alias_name(&title)
                        .filter(|name| *name != id)
                        .map(|alias| {
                            let attr = handler.node().attr(fs).expect("attr should be available");
                            let handler = fs.node_alloc(|ino| Node::new_link(ino, &id, &attr));
                            (alias, handler)
                        });

                entries.insert(
                    id,
                    SecretHandler {
                        node: handler,
                        alias: alias_handler,
                    },
                );
            }

            Ok(())
        })?;

        Ok(entries
            .iter()
            .flat_map(|(name, handler)| {
                let entry = DirEntry {
                    inode: handler.node.ino(),
                    name: name.clone(),
                    file_type: FileType::Directory,
                };
                match &handler.alias {
                    Some((alias, handler)) => vec![
                        entry,
                        DirEntry {
                            inode: handler.ino(),
                            name: alias.clone(),
                            file_type: FileType::Symlink,
                        },
                    ],
                    None => vec![entry],
                }
            })
            .map(|entry| (entry.name.clone(), entry))
            .collect::<HashMap<String, DirEntry>>() // Deduplicate entries
            .into_values())
    }
}

/// Creates the file attributes of a vault node.
fn make_attr(node: &Vault, fs: &Fs) -> FileAttr {
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

/// Creates an alias name for the title.
fn make_alias_name(title: &str) -> Option<String> {
    Some(title.replace('/', "_")).filter(|alias| !alias.is_empty())
}
