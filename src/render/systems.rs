//! A module that defines all systems responsible for rendering the UI.

use std::time::Instant;

use bevy::asset::{AssetLoadFailedEvent, LoadState};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::asset::NekoMaidUI;
use crate::components::{NekoUINode, NekoUITree, ScopeNotificationMap};
use crate::marker::MarkerRegistry;
use crate::parse::element::NekoElementBuilder;
use crate::parse::scope::ScopeId;
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

        root.scope = asset.scope.clone();
        for name in asset.scope.dependency_graph().nodes() {
            root.update_names.insert(name.clone());
        }
        root.scope_notification.clear();

        for element in &asset.elements {
            spawn_element(
                &asset_server,
                &markers,
                &mut root.scope_notification,
                &mut commands,
                &element,
                root_entity,
                root_entity,
            );
        }

        debug!(
            "Spawned tree {root_entity} in {} ms.",
            t.elapsed().as_millis()
        );
    }
}

/// Recursively spawns a [`NekoElementBuilder`] and its children.
fn spawn_element(
    asset_server: &Res<AssetServer>,
    markers: &MarkerRegistry,
    scope_notification: &mut ScopeNotificationMap,
    commands: &mut Commands,
    element: &NekoElementBuilder,
    parent: Entity,
    root: Entity,
) {
    let entity =
        (element.native_widget.spawn_func)(asset_server, commands, &element.element, parent);

    markers.insert(commands.entity(entity), &element.element);
    scope_notification.register(element.element.scope_id(), entity);

    // FIX this actually should be executed every time there is a class update
    for style in element.element.active_styles() {
        scope_notification.register(style.scope_id, entity);
    }

    commands.entity(entity).insert((NekoUINode {
        root,
        element: element.element.clone(),
        updated_properties: vec![],
    },));

    for child in &element.children {
        spawn_element(asset_server, markers, scope_notification, commands, child, entity, root);
    }
}

/// Update scope of Neko UI trees.
pub fn update_scope(
    mut roots: Query<(Entity, &mut NekoUITree), Changed<NekoUITree>>,
    mut nodes: Query<&mut NekoUINode>,
) {
    for (entity, root) in roots.iter_mut() {
        let t = Instant::now();

        let root = root.into_inner();
        let scopes = &mut root.scope;

        let Some(global_scope) = scopes.get_mut(ScopeId(0)) else {
            return;
        };
        
        let update_names = &root.update_names;
        if update_names.is_empty() {
            return;
        }
        global_scope.add_resolved_variables(root.variables.iter());

        let variables = {
            let graph = scopes.dependency_graph();

            let mut to_update = HashSet::new();
            let mut remaining = update_names.iter().collect::<Vec<_>>();
            while let Some(name) = remaining.pop() {
                to_update.insert(name);
                remaining.extend(graph.get_dependents(name));
            }

            let mut variables = to_update.iter().map(|&n| n.clone()).collect::<Vec<_>>();
            let order = graph.order_map();
            variables.sort_by_key(|n| order.get(n).unwrap_or(&0));

            variables
        };

        // println!(
        //     "Updating variables: {}",
        //     variables
        //         .iter()
        //         .map(|v| format!("{v}"))
        //         .collect::<Vec<_>>()
        //         .join(", ")
        // );

        for name in &variables {
            scopes.evaluate(name);

            for entity in root.scope_notification.get(name.scope_id()) {
                let Ok(mut node) = nodes.get_mut(entity) else { continue };
                node.updated_properties.push(name.name().clone());
            }
        }

        root.update_names.clear();

        debug!(
            "Updated scope of {entity} in {} ms.",
            t.elapsed().as_millis()
        );
    }
}

/// Update node properties.
pub(crate) fn update_nodes(
    asset_server: Res<AssetServer>,
    mut roots: Query<&mut NekoUITree>,
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
    if q.is_empty() { return }

    let t = Instant::now();

    for (
        neko_node,
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
        let NekoUINode {
            updated_properties,
            element,
            root,
            ..
        } = neko_node.into_inner();

        let Ok(mut root) = roots.get_mut(*root) else {
            continue;
        };

        update_node(
            &asset_server,
            element.view_mut(&mut root.scope),
            updated_properties.iter(),
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

        updated_properties.clear();
    }

    debug!(
        "Updated node properties in {} ms.",
        t.elapsed().as_millis()
    );
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
