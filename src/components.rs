//! Components used for the NekoMaid plugin.

use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use lazy_static::lazy_static;

use crate::asset::NekoMaidUI;
use crate::parse::element::NekoElement;
use crate::parse::scope::{ScopeId, ScopeName, ScopeTree};
use crate::parse::value::PropertyValue;

/// A component representing a node of a NekoMaid UI tree.
#[derive(Component)]
pub struct NekoUINode {
    /// The entity with the NekoUITree component.
    pub root: Entity,
    /// The element struct that this node renders.
    pub element: NekoElement,
    /// A list of properties that changed and need to be re-rendered.
    pub updated_properties: Vec<String>,
}

lazy_static! {
    static ref EMPTY_SET: HashSet<Entity> = HashSet::new();
}

/// 
#[derive(Debug, Deref, DerefMut, Default)]
pub(crate) struct ScopeNotificationMap {
    #[deref]
    map: HashMap<ScopeId, HashSet<Entity>>
}
impl ScopeNotificationMap {
    /// Register a node entity as listener to the scope specified.
    pub fn register(&mut self, scope: ScopeId, entity: Entity) {
        self.map.entry(scope).or_default().insert(entity);
    }

    /// Returns an iterator of node entities that listen to changes in the given scope.
    pub fn get(&self, scope: ScopeId) -> impl Iterator<Item=Entity> {
        self.map.get(&scope).unwrap_or(&EMPTY_SET).iter().cloned()
    }
}

/// A component representing the root of a NekoMaid UI tree.
#[derive(Debug, Component)]
#[require(Node)]
pub struct NekoUITree {
    /// The NekoMaid UI asset associated with this tree.
    asset: Handle<NekoMaidUI>,

    /// Whether the tree needs to be re-spawned.
    dirty: bool,

    /// Variables that should be inserted into the global context.
    pub(crate) variables: HashMap<String, PropertyValue>,

    /// The scope tree used to render elements from this tree.
    pub(crate) scope: ScopeTree,

    /// Scope names to update.
    pub(crate) update_names: HashSet<ScopeName>,

    /// A map to trigger node updates when a targetted scope changes.
    pub(crate) scope_notification: ScopeNotificationMap,
}

impl NekoUITree {
    /// Creates a new NekoUITree with the given asset handle.
    pub fn new(asset: Handle<NekoMaidUI>) -> Self {
        Self {
            asset,
            variables: HashMap::new(),
            dirty: true,
            scope: ScopeTree::default(),
            update_names: HashSet::new(),
            scope_notification: ScopeNotificationMap::default(),
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
        for (name, value) in variables {
            self.set_variable(&name, value);
        }
        self
    }

    /// Sets a variable to the specified value.
    pub fn set_variable(&mut self, name: &str, value: PropertyValue) {
        self.variables.insert(name.to_owned(), value);
        self.update_names.insert(ScopeName::Variable(name.to_owned(), ScopeId(0)));
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
