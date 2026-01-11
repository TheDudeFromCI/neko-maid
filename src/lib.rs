#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::prelude::*;

use crate::asset::{NekoMaidAssetLoader, NekoMaidUI};
use crate::marker::{MarkerAppExt, MarkerRegistry};
use crate::render::systems::{self, removed_interactable};

pub mod asset;
pub mod components;
pub mod marker;
pub mod native;
pub mod parse;
pub mod render;

/// A Bevy UI plugin: NekoMaid
///
/// This plugin provides core functionality for the NekoMaid framework,
/// including UI components and systems, assets, and high-level widgets.
pub struct NekoMaidPlugin;
impl Plugin for NekoMaidPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_asset::<NekoMaidUI>()
            .init_asset_loader::<NekoMaidAssetLoader>()
            .init_resource::<MarkerRegistry>()
            .add_marker::<Interaction>()
            .add_observer(removed_interactable)
            .add_systems(
                Update,
                (
                    (
                        systems::spawn_tree,
                        systems::handle_interactions,
                        systems::handle_class_changes,
                        systems::update_styles,
                        systems::update_scope,
                        systems::update_nodes,
                    )
                        .chain()
                        .in_set(NekoMaidSystems::UpdateTree),
                    systems::update_tree.in_set(NekoMaidSystems::AssetListener),
                    systems::asset_failure.in_set(NekoMaidSystems::AssetListener),
                ),
            )
            .configure_sets(
                Update,
                NekoMaidSystems::AssetListener.before(NekoMaidSystems::UpdateTree),
            );
    }
}

/// System sets used by the NekoMaid plugin.
#[derive(Debug, SystemSet, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NekoMaidSystems {
    /// System for spawning UI trees.
    UpdateTree,

    /// System for listening for asset changes.
    AssetListener,
}
