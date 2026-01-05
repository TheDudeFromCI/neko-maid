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

use crate::parse::element::NekoElement;

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

/// The marker factory.
pub type MarkerFactory = Box<dyn Fn(&mut EntityCommands) + Send + Sync>;

/// A resource for managing registered marker types.
#[derive(Default, Resource)]
pub struct MarkerRegistry {
    /// Maps marker names to marker factories.
    factories: HashMap<String, MarkerFactory>,
}

impl MarkerRegistry {
    /// Inserts the marker component to an entity given its element.
    pub fn insert(&self, mut entity: EntityCommands, element: &NekoElement) {
        for class in element.classes() {
            let Some(f) = self.factories.get(class) else {
                continue;
            };
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
            .factories
            .insert(
                T::id().to_owned(),
                Box::new(|entity| {
                    entity.insert(T::new());
                }),
            );

        self
    }
}
