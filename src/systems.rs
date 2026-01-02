//! Systems used by the NekoMaid plugin.

use bevy::asset::{AssetLoadFailedEvent, LoadState};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::asset::NekoMaidUI;
use crate::components::NekoUITree;
use crate::parse::context::NekoResult;
use crate::parse::element::NekoElementBuilder;
use crate::parse::value::PropertyValue;

/// Listens for changes to the [`NekoUITree`] component and spawns the UI tree
/// accordingly.
#[allow(clippy::type_complexity)]
pub(super) fn spawn_tree(
    asset_server: Res<AssetServer>,
    assets: Res<Assets<NekoMaidUI>>,
    mut roots: Query<
        (Entity, &mut NekoUITree, &mut Node),
        Or<(Added<NekoUITree>, Changed<NekoUITree>)>,
    >,
    mut commands: Commands,
) {
    for (entity, mut root, mut node) in roots.iter_mut() {
        if !root.is_dirty() {
            continue;
        }

        root.clear_dirty();
        commands.entity(entity).despawn_children();

        *node = Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        };

        let Some(asset) = assets.get(root.asset()) else {
            match asset_server.get_load_state(root.asset()) {
                Some(LoadState::Loading) => {}
                _ => error!("Failed to load NekoMaidUI asset for NekoUITree"),
            }
            continue;
        };

        let mut variables = root.variables().clone();
        for (name, unresolved) in &asset.variables {
            if variables.contains_key(name) {
                continue;
            }

            if let Ok(v) = unresolved.resolve(&variables) {
                variables.insert(name.clone(), v);
            }
        }

        for element in &asset.elements {
            let mut element = element.clone();
            if let Err(e) = resolve_scope(&mut element, &variables) {
                error!("{}", e);
            }
            spawn_element(&asset_server, &mut commands, &element, entity);
        }
    }
}

/// Resolve variable scope
pub fn resolve_scope(
    element: &mut NekoElementBuilder,
    variables: &HashMap<String, PropertyValue>,
) -> NekoResult<()> {
    element.element.resolve(variables)?;

    let mut variables = variables.clone();
    for (name, value) in element.element.properties() {
        variables.insert(name.clone(), value.clone());
    }

    for child in &mut element.children {
        resolve_scope(child, &variables)?;
    }

    Ok(())
}

/// Recursively spawns a [`NekoElementBuilder`] and its children.
fn spawn_element(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    element: &NekoElementBuilder,
    parent: Entity,
) {
    let entity =
        (element.native_widget.spawn_func)(asset_server, commands, &element.element, parent);

    for child in &element.children {
        spawn_element(asset_server, commands, child, entity);
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

/// Listens for asset load failures and clears any existing UI trees that
/// reference the failed asset.
///
/// (Having a UI tree suddenly disappear is a good indicator to the developer
/// that something has gone wrong with their code.)
pub(super) fn asset_failure(
    mut asset_failures: MessageReader<AssetLoadFailedEvent<NekoMaidUI>>,
    mut roots: Query<&mut NekoUITree>,
) {
    for event in asset_failures.read() {
        for mut root in roots.iter_mut() {
            if root.asset().id() == event.id {
                root.mark_dirty();
            }
        }
    }
}
