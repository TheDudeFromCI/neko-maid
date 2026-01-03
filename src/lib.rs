#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::prelude::*;

use crate::{asset::{NekoMaidAssetLoader, NekoMaidUI}, marker::MarkerRegistry};

pub mod asset;
pub mod components;
pub mod native;
pub mod parse;
pub mod marker;
mod systems;

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
            .add_systems(
                Update,
                (
                    systems::spawn_tree.in_set(NekoMaidSystems::UpdateTree),
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
