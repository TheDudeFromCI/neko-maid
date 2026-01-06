//! A module that defines all systems responsible for rendering the UI.

use std::time::Instant;

use bevy::asset::{AssetLoadFailedEvent, LoadState};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::asset::NekoMaidUI;
use crate::components::{NekoUINode, NekoUITree};
use crate::marker::MarkerRegistry;
use crate::parse::context::NekoResult;
use crate::parse::element::NekoElementBuilder;
use crate::parse::value::PropertyValue;
use crate::render::update::update_node;

/// Listens for changes to the [`NekoUITree`] component and spawns the UI tree
/// accordingly.
#[allow(clippy::type_complexity)]
pub(crate) fn spawn_tree(
    asset_server: Res<AssetServer>,
    assets: Res<Assets<NekoMaidUI>>,
    markers: Res<MarkerRegistry>,
    mut roots: Query<
        (Entity, &mut NekoUITree, &mut Node),
        Or<(Added<NekoUITree>, Changed<NekoUITree>)>,
    >,
    mut commands: Commands,
) {
    for (root_entity, mut root, mut node) in roots.iter_mut() {
        if !root.is_dirty() {
            continue;
        }
        let t = Instant::now();

        root.clear_dirty();
        commands.entity(root_entity).despawn_children();

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
            spawn_element(
                &asset_server,
                &markers,
                &mut commands,
                &element,
                root_entity,
                root_entity,
            );
        }

        info!(
            "Spawned tree {root_entity} in {} ms.",
            t.elapsed().as_millis()
        );
    }
}

/// Recursively spawns a [`NekoElementBuilder`] and its children.
fn spawn_element(
    asset_server: &Res<AssetServer>,
    markers: &MarkerRegistry,
    commands: &mut Commands,
    element: &NekoElementBuilder,
    parent: Entity,
    root: Entity,
) {
    let entity =
        (element.native_widget.spawn_func)(asset_server, commands, &element.element, parent);

    markers.insert(commands.entity(entity), &element.element);

    commands.entity(entity).insert((NekoUINode {
        element: element.element.clone(),
        updated_properties: element
            .element
            .active_properties()
            .cloned()
            .collect::<Vec<_>>(),
        root,
    },));

    for child in &element.children {
        spawn_element(asset_server, markers, commands, child, entity, root);
    }
}

/// Resolve variable scope
pub(crate) fn resolve_scope(
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

/// Update node properties.
pub(crate) fn update_nodes(
    asset_server: Res<AssetServer>,
    q: Query<
        (
            &mut NekoUINode,
            &mut Node,
            &mut BorderColor,
            &mut BorderRadius,
            &mut BackgroundColor,
            Option<&mut ImageNode>,
            Option<&mut Text>,
            Option<&mut TextSpan>,
            Option<&mut TextFont>,
            Option<&mut TextColor>,
            Option<&mut TextLayout>,
        ),
        Changed<NekoUINode>,
    >,
) {
    for (
        mut neko_node,
        mut node,
        mut border_color,
        mut border_radius,
        mut background_color,
        image_node,
        text,
        span,
        font,
        color,
        layout,
    ) in q
    {
        // println!("Updating properties {:?} from {entity}",
        // neko_node.updated_properties);
        let properties = neko_node.updated_properties.iter();

        update_node(
            &asset_server,
            &neko_node.element,
            properties,
            &mut node,
            &mut border_color,
            &mut border_radius,
            &mut background_color,
            &mut image_node.map(|v| v.into_inner()),
            &mut text.map(|v| v.into_inner()),
            &mut span.map(|v| v.into_inner()),
            &mut font.map(|v| v.into_inner()),
            &mut color.map(|v| v.into_inner()),
            &mut layout.map(|v| v.into_inner()),
        );

        neko_node.updated_properties.clear();
    }
}

/// Listens for changes to the [`NekoMaidUI`] asset and updates any existing UI
/// trees accordingly.
pub(crate) fn update_tree(
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
pub(crate) fn asset_failure(
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
