//! Components used for the NekoMaid plugin.

use bevy::prelude::*;

use crate::asset::NekoMaidUI;

/// A component representing the root of a NekoMaid UI tree.
#[derive(Debug, Component)]
#[require(Node)]
pub struct NekoUITree {
    /// The NekoMaid UI asset associated with this tree.
    asset: Handle<NekoMaidUI>,

    /// Whether the tree needs to be re-spawned.
    dirty: bool,
}

impl NekoUITree {
    /// Creates a new NekoUITree with the given asset handle.
    pub fn new(asset: Handle<NekoMaidUI>) -> Self {
        Self { asset, dirty: true }
    }

    /// Returns a reference to the asset handle of this tree.
    pub fn asset(&self) -> &Handle<NekoMaidUI> {
        &self.asset
    }

    /// Marks the tree as dirty, indicating that it needs to be re-spawned.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clears the dirty flag.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Returns whether the tree is dirty.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}
