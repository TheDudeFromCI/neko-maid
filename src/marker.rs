//! This module implements the class marker functionality.
//!
//! Class markers are components that are automatically attached to
//! UI nodes that have the associated class. Given the `MyMarker` component
//! defined like below,
//!
//! ```
//! // define the marker component
//!
//! #[derive(Component, NekoMarker)]
//! #[neko_marker("my_marker")]
//! pub struct MyMarker;
//!
//! // register the marker type.
//!
//! app.add_marker::<MyMarker>();
//! ```
//!
//! All layout nodes with the `my_marker` class will have the `MyMarker`
//! component.
//!
//! ```
//! layout div {
//!     class my_marker;
//! }
//! ```

use bevy::app::App;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::EntityCommands;
use bevy::platform::collections::HashMap;
use bevy::ui::Interaction;
pub use neko_derive::NekoMarker;

/// The marker trait. It can easily be implemented with derive.
///
/// ```
/// #[derive(Component, NekoMarker)]
/// #[neko_marker("my_marker")]
/// pub struct MyMarker;
/// ```
pub trait NekoMarker: 'static {
    /// Create a new instance of the marker
    fn new() -> Self
    where
        Self: Sized;

    /// Return the marker id
    fn id() -> &'static str
    where
        Self: Sized;
}

// Makes elements optionally interactable through the `interactable` class.
impl NekoMarker for Interaction {
    fn new() -> Self
    where
        Self: Sized,
    {
        Interaction::default()
    }

    fn id() -> &'static str
    where
        Self: Sized,
    {
        "interactable"
    }
}

/// The marker insert/remove function.
pub type MarkerFunction = Box<dyn Fn(&mut EntityCommands) + Send + Sync>;

/// A resource for managing registered marker types.
#[derive(Default, Resource)]
pub struct MarkerRegistry {
    /// Maps marker names to marker inserters.
    inserters: HashMap<String, Vec<MarkerFunction>>,
    /// Maps marker names to marker removers.
    removers: HashMap<String, Vec<MarkerFunction>>,
}

impl MarkerRegistry {
    /// Registers the specified marker component.
    pub fn add_marker<T: NekoMarker + Bundle>(&mut self) {
        self.inserters
            .entry(T::id().to_owned())
            .or_default()
            .push(Box::new(|entity| {
                entity.insert(T::new());
            }));
        self.removers
            .entry(T::id().to_owned())
            .or_default()
            .push(Box::new(|entity| {
                entity.remove::<T>();
            }));
    }

    /// Inserts the associated class marker components to the node entity.
    pub fn insert(&self, mut entity: EntityCommands, class: &str) {
        let Some(inserters) = self.inserters.get(class) else {
            return;
        };
        for f in inserters {
            f(&mut entity);
        }
    }

    /// Removes the associated class marker components from the node entity.
    pub fn remove(&self, mut entity: EntityCommands, class: &str) {
        let Some(removers) = self.removers.get(class) else {
            return;
        };
        for f in removers {
            f(&mut entity);
        }
    }
}

/// A trait to easily register types that implement the [NekoMarker] trait.
///
/// ```
/// app.add_marker::<MyMarker>();
/// ```
pub trait MarkerAppExt {
    /// Registers a marker type.
    fn add_marker<T: NekoMarker + Bundle>(&mut self) -> &mut Self;
}

impl MarkerAppExt for App {
    fn add_marker<T: NekoMarker + Bundle>(&mut self) -> &mut Self {
        self.init_resource::<MarkerRegistry>()
            .world_mut()
            .resource_mut::<MarkerRegistry>()
            .add_marker::<T>();
        self
    }
}
