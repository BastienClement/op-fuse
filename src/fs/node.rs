use std::{cell::RefCell, rc::Rc};

use fuser::FileAttr;

use crate::{
    onepassword::{id, types::SecretMetadata},
    util::SharedCell,
};

use self::secret::FieldValue;

use super::{Fs, Inode};

pub mod account;
pub mod field;
pub mod link;
pub mod root;
pub mod secret;
pub mod vault;

/// A node in the filesystem tree.
pub enum Node {
    /// A dummy node. Used as a placeholder for inode 0.
    /// Also returned when a node is not found.
    Dummy,

    /// The root node.
    Root(Box<root::Root>),

    /// An account node. This is the first level of the tree.
    Account(Box<account::Account>),

    /// A vault node. This is the second level of the tree.
    Vault(Box<vault::Vault>),

    /// A secret node. This is the third level of the tree.
    Secret(Box<secret::Secret>),

    /// A secret-field node. This is the fourth level of the tree.
    Field(Box<field::Field>),

    /// A link node. This is a symlink to another node.
    Link(Box<link::Link>),
}

impl Node {
    /// Creates a new dummy node.
    pub fn new_dummy() -> Node {
        Node::Dummy
    }

    /// Creates a new root node.
    pub fn new_root(fs: &Fs) -> Node {
        Node::Root(Box::new(root::Root::new(fs)))
    }

    /// Creates a new account node.
    pub fn new_account(ino: Inode, id: id::Account) -> Node {
        Node::Account(Box::new(account::Account::new(ino, id)))
    }

    /// Creates a new vault node.
    pub fn new_vault(ino: Inode, id: id::Vault) -> Node {
        Node::Vault(Box::new(vault::Vault::new(ino, id)))
    }

    /// Creates a new secret node.
    pub fn new_secret(ino: Inode, id: id::Secret, meta: SecretMetadata) -> Node {
        Node::Secret(Box::new(secret::Secret::new(ino, id, meta)))
    }

    /// Creates a new field node.
    pub fn new_field(
        ino: Inode,
        metadata: SharedCell<SecretMetadata>,
        data: SharedCell<FieldValue>,
        trim: bool,
    ) -> Node {
        Node::Field(Box::new(field::Field::new(ino, metadata, data, trim)))
    }

    /// Creates a new link node.
    pub fn new_link(ino: Inode, target: &str, attr: &FileAttr) -> Node {
        Node::Link(Box::new(link::Link::new(ino, target, attr)))
    }

    /// Returns the filesystem attributes of the node.
    /// Returns `None` if the node is a dummy node.
    pub fn attr(&self, fs: &Fs) -> Option<FileAttr> {
        Some(match self {
            Node::Dummy => return None,
            Node::Root(node) => node.attr(),
            Node::Account(node) => node.attr(fs),
            Node::Vault(node) => node.attr(fs),
            Node::Secret(node) => node.attr(fs),
            Node::Field(node) => node.attr(fs),
            Node::Link(node) => node.attr(),
        })
    }
}

/// A slab of nodes.
struct Slab {
    inner: RefCell<::slab::Slab<Rc<Node>>>,
}

impl Slab {
    /// Allocates a new node and returns its inode.
    fn alloc<F>(&self, node: F) -> Inode
    where
        F: FnOnce(Inode) -> Node,
    {
        let mut nodes = self.inner.borrow_mut();
        let entry = nodes.vacant_entry();

        let ino = entry.key() as Inode;
        entry.insert(Rc::new(node(ino)));
        ino
    }

    /// Gets a node by its inode.
    fn get(&self, ino: Inode) -> Rc<Node> {
        // To prevent panics, this must not return the borrowed ref
        self.inner
            .borrow()
            .get(super::u64_to_usize(ino))
            .map_or_else(|| Rc::new(Node::new_dummy()), Rc::clone)
    }

    /// Removes a node by its inode.
    fn free(&self, ino: Inode) {
        self.inner.borrow_mut().remove(super::u64_to_usize(ino));
    }
}

/// A set of nodes.
/// This is a wrapper around a slab of nodes and handles node lifetime.
pub struct Set {
    slab: Rc<Slab>,
}

impl Set {
    /// Creates a new set of nodes.
    pub fn new() -> Set {
        Set {
            slab: Rc::new(Slab {
                inner: RefCell::new(::slab::Slab::new()),
            }),
        }
    }

    /// Allocates a new node and returns a handler to it.
    /// The handler will free the node when dropped, unless `persist` is called.
    pub fn alloc<F>(&self, node: F) -> Handler
    where
        F: FnOnce(Inode) -> Node,
    {
        let ino = self.slab.alloc(node);
        Handler(ino, Some(self.slab.clone()))
    }

    /// Gets a node by its inode.
    pub fn get(&self, ino: Inode) -> Rc<Node> {
        self.slab.get(ino)
    }
}

/// A handler for a node.
/// It frees the node when dropped, unless `persist` is called.
pub struct Handler(Inode, Option<Rc<Slab>>);

impl Handler {
    /// Returns the inode of the node.
    pub fn ino(&self) -> Inode {
        self.0
    }

    /// Returns a reference to the node.
    pub fn node(&self) -> Rc<Node> {
        self.1
            .as_ref()
            .expect("the slab cannot be None")
            .get(self.0)
            .clone()
    }

    /// Prevents the node from being freed when dropped.
    /// Consumes the handler and returns the inode.
    pub fn persist(mut self) -> Inode {
        self.1 = None;
        self.0
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        if let Some(slab) = self.1.take() {
            debug!(ino = self.0, "freeing node");
            slab.free(self.0);
        }
    }
}
