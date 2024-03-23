use fuser::FileAttr;

use crate::{
    fs::{Fs, Inode},
    onepassword::types::SecretMetadata,
    util::SharedCell,
};

use super::secret::FieldValue;

/// A node representing a secret field.
pub struct Field {
    /// The inode number of the node.
    ino: Inode,

    /// The metadata of the secret.
    metadata: SharedCell<SecretMetadata>,

    /// The data of the field.
    data: SharedCell<FieldValue>,

    /// Whether to trim the field value.
    ///
    /// If true, the field value is trimmed to remove the first three backticks.
    /// This is a workaround for the fact that the note field might be stored as
    /// a markdown code block.
    trim: bool,
}

impl Field {
    /// Creates a new field node.
    pub fn new(
        ino: Inode,
        metadata: SharedCell<SecretMetadata>,
        data: SharedCell<FieldValue>,
        trim: bool,
    ) -> Field {
        Self {
            ino,
            metadata,
            data,
            trim,
        }
    }

    /// Returns the file attributes of the node.
    pub fn attr(&self, fs: &Fs) -> FileAttr {
        let metadata = self.metadata.borrow();
        let data = self.data.borrow();

        let updated_at = metadata.updated_at.into();
        let created_at = metadata.created_at.into();

        FileAttr {
            ino: self.ino,
            size: data.value().len() as u64,
            blocks: 0,
            atime: updated_at,
            mtime: updated_at,
            ctime: updated_at,
            crtime: created_at,
            kind: fuser::FileType::RegularFile,
            perm: fs.config.file_mode,
            nlink: 1,
            uid: fs.config.uid,
            gid: fs.config.gid,
            rdev: 0,
            flags: 0,
            blksize: 512,
        }
    }

    /// Reads the field value.
    pub fn read(&self, offset: usize, size: usize) -> Vec<u8> {
        use std::cmp::min;

        let data = self.data.borrow();
        let mut value = data.value();
        if self.trim && value.starts_with("```") {
            value = &value[3..];
        }
        let bytes = value.as_bytes();

        let start = min(bytes.len(), offset);
        let end = min(bytes.len(), offset + size);

        Vec::from(&bytes[start..end])
    }
}
