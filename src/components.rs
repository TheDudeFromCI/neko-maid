//! Components used for the NekoMaid plugin.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::asset::NekoMaidUI;
use crate::parse::value::PropertyValue;

/// A component representing the root of a NekoMaid UI tree.
#[derive(Debug, Component)]
#[require(Node)]
pub struct NekoUITree {
    /// The NekoMaid UI asset associated with this tree.
    asset: Handle<NekoMaidUI>,

    /// Whether the tree needs to be re-spawned.
    dirty: bool,

    /// Variables that should be inserted into the global context.
    variables: HashMap<String, PropertyValue>,
}

impl NekoUITree {
    /// Creates a new NekoUITree with the given asset handle.
    pub fn new(asset: Handle<NekoMaidUI>) -> Self {
        Self {
            asset,
            variables: HashMap::new(),
            dirty: true,
        }
    }

    /// Returns a reference to the asset handle of this tree.
    pub fn asset(&self) -> &Handle<NekoMaidUI> {
        &self.asset
    }

    /// Returns a reference to the variable map.
    pub fn variables(&self) -> &HashMap<String, PropertyValue> {
        &self.variables
    }

    /// Extends the defined variables.
    pub fn with_variables(mut self, variables: HashMap<String, PropertyValue>) -> Self {
        self.variables.extend(variables);
        self
    }

    /// Sets a variable to the specified value.
    pub fn set_variable(&mut self, name: &str, value: PropertyValue) {
        self.variables.insert(name.to_owned(), value);
        self.mark_dirty();
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
