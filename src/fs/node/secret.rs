use std::{cell::RefCell, collections::HashMap};

use anyhow::Result;
use fuser::{FileAttr, FileType};

use crate::{
    fs::{dentry::DirEntry, Fs, Inode},
    onepassword::{id, types::SecretMetadata},
    util::{diff, SharedCell, Throttle},
};

use super::{Handler, Node};

/// A secret node.
pub struct Secret {
    /// The inode number of the node.
    ino: Inode,

    /// The ID of the secret.
    id: id::Secret,

    /// The metadata of the secret.
    metadata: SharedCell<SecretMetadata>,

    /// The field handlers of the secret.
    entries: RefCell<Throttle<HashMap<String, FieldHandler>>>,
}

/// A field handler that contains the field-node handler, the alias handler if
/// present and the field data.
pub struct FieldHandler {
    /// The field-node handler.
    node: Handler,

    /// The alias handler if present.
    /// If present, the alias is a symlink to the field-node.
    alias: Option<(String, Handler)>,

    /// The field data.
    data: SharedCell<FieldValue>,
}

/// The value of a field.
pub struct FieldValue(String);

impl FieldValue {
    /// Returns the value of the field.
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl Secret {
    /// Creates a new secret node.
    pub fn new(ino: Inode, id: id::Secret, meta: SecretMetadata) -> Secret {
        Self {
            ino,
            id,
            metadata: SharedCell::new(meta),
            entries: RefCell::new(Throttle::default()),
        }
    }

    /// Returns the file attributes of the node.
    pub fn attr(&self, fs: &Fs) -> FileAttr {
        let meta = self.metadata.borrow();
        let updated_at = meta.updated_at.into();
        let created_at = meta.created_at.into();
        FileAttr {
            ino: self.ino,
            size: 512,
            blocks: 0,
            atime: updated_at,
            mtime: updated_at,
            ctime: updated_at,
            crtime: created_at,
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

    /// Updates the metadata of the secret.
    ///
    /// This is used to update the metadata of the secret when the full vault
    /// is refreshed.
    pub fn update_metadata(&self, meta: SecretMetadata) {
        *self.metadata.borrow_mut() = meta;
    }

    /// Returns the directory entries of the node.
    pub fn entries(&self, fs: &Fs) -> Result<impl Iterator<Item = DirEntry>> {
        let mut entries = self.entries.borrow_mut();

        entries.try_refresh(fs.config.cache_duration, |entries| {
            let secret = fs.op.get_secret(&self.id)?;
            self.update_metadata(secret.metadata);

            let mut fields = secret
                .fields
                .into_iter()
                .filter(|field| !field.id.is_empty())
                .map(|field| (field.id.clone(), field))
                .collect::<HashMap<_, _>>();

            let (delete, update, create) = diff(entries.keys(), fields.keys());

            for id in delete {
                entries.remove(&id);
            }

            for id in update {
                let field = fields.remove(&id).expect("field should be in list");
                let handler = entries.get(&id).expect("handler should be in list");

                handler.data.borrow_mut().0 = field.value.unwrap_or_default();
            }

            for id in create {
                let field = fields.remove(&id).expect("field should be in list");

                let data = SharedCell::new(FieldValue(field.value.unwrap_or_default()));
                let node = fs.node_alloc({
                    let metadata = self.metadata.clone();
                    let data = data.clone();
                    let trim = id == "notesPlain"; // Trim the notes field only
                    move |ino| Node::new_field(ino, metadata, data, trim)
                });

                let alias = field_alias(&field.reference)
                    .filter(|alias| *alias != field.id)
                    .map(|alias| {
                        let attr = node.node().attr(fs).expect("attr should be available");
                        let handler = fs.node_alloc(|ino| Node::new_link(ino, &field.id, &attr));
                        (alias, handler)
                    });

                entries.insert(id, FieldHandler { node, alias, data });
            }

            Ok(())
        })?;

        Ok(entries
            .iter()
            .flat_map(|(name, handler)| {
                let main = DirEntry {
                    inode: handler.node.ino(),
                    name: name.clone(),
                    file_type: FileType::RegularFile,
                };
                match &handler.alias {
                    Some((alias, handler)) => vec![
                        main,
                        DirEntry {
                            inode: handler.ino(),
                            name: alias.clone(),
                            file_type: FileType::Symlink,
                        },
                    ],
                    None => vec![main],
                }
            })
            .collect::<Vec<DirEntry>>()
            .into_iter())
    }
}

/// Returns the alias name of a field.
///
/// The alias name is made from the last parts of the reference.
///
/// It is only generated if the reference is not empty and does not end with a
/// slash, as this might be the case when the `title` is broken.
fn field_alias(reference: &str) -> Option<String> {
    if reference.is_empty() || reference.ends_with('/') {
        return None;
    }

    // We only need the last parts of the reference to get the item and field.
    // The reference is in the format:
    //
    //     op://<account>/<vault>/<item>/<field>
    //       1 2         3       4      5
    //
    let parts = reference.split('/').skip(4);
    Some(parts.collect::<Vec<&str>>().join("_"))
}
