//! Systems used by the NekoMaid plugin.

use bevy::asset::LoadState;
use bevy::prelude::*;

use crate::asset::NekoMaidUI;
use crate::components::NekoUITree;
use crate::parse::element::NekoElementBuilder;

/// Listens for changes to the [`NekoUITree`] component and spawns the UI tree
/// accordingly.
#[allow(clippy::type_complexity)]
pub(super) fn spawn_tree(
    asset_server: Res<AssetServer>,
    assets: Res<Assets<NekoMaidUI>>,
    mut roots: Query<(Entity, &mut NekoUITree), Or<(Added<NekoUITree>, Changed<NekoUITree>)>>,
    mut commands: Commands,
) {
    for (entity, mut root) in roots.iter_mut() {
        if !root.is_dirty() {
            continue;
        }

        root.clear_dirty();
        commands.entity(entity).despawn_children();

        let Some(asset) = assets.get(root.asset()) else {
            match asset_server.get_load_state(root.asset()) {
                Some(LoadState::Loading) => {}
                _ => error!("Failed to load NekoMaidUI asset for NekoUITree"),
            }
            continue;
        };

        for element in &asset.elements {
            spawn_element(&mut commands, element, entity);
        }
    }
}

/// Recursively spawns a [`NekoElementBuilder`] and its children.
fn spawn_element(commands: &mut Commands, element: &NekoElementBuilder, parent: Entity) {
    let entity = (element.native_widget.spawn_func)(commands, parent, &element.element);

    for child in &element.children {
        spawn_element(commands, child, entity);
    }
}

/// Listens for changes to the [`NekoMaidUI`] asset and updates any existing UI
/// trees accordingly.
pub(super) fn update_tree(
    mut asset_updates: MessageReader<AssetEvent<NekoMaidUI>>,
    mut roots: Query<&mut NekoUITree>,
) {
    for event in asset_updates.read() {
        match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => {
                for mut root in roots.iter_mut() {
                    if root.asset().id() == *id {
                        root.mark_dirty();
                    }
                }
            }
            _ => {}
        }
    }
}
