//! A module for implementing scoping rules for variables and properties.

use std::fmt::{Display, Write};

use bevy::ecs::entity::Entity;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{Deref, DerefMut};
use lazy_static::lazy_static;

use crate::parse::property::UnresolvedPropertyValue;
use crate::parse::value::PropertyValue;

/// An entry in a scope.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScopeItem {
    /// The unresolved expression/value to be evaluated.
    pub unresolved: UnresolvedPropertyValue,
    /// The evaluated value, either derived from the `unresolved`
    /// property or injected directly.
    pub value: Option<PropertyValue>,
}

/// The scope id based on its index in the scope tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deref)]
pub(crate) struct ScopeId(pub usize);

/// An uniquely defined name in a scope tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ScopeName {
    Variable(String, ScopeId),
    Property(String, ScopeId),
}
impl ScopeName {
    /// Returns the property or variable name of this scope name.
    pub fn name(&self) -> &String {
        match self {
            ScopeName::Variable(name, _) => name,
            ScopeName::Property(name, _) => name,
        }
    }
    
    /// Returns the id of the scope that owns this scope name.
    pub fn scope_id(&self) -> ScopeId {
        match self {
            ScopeName::Variable(_, scope_id) => *scope_id,
            ScopeName::Property(_, scope_id) => *scope_id,
        }
    }
}
impl Display for ScopeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopeName::Variable(name, scope_id) => write!(f, "${}#{}", name, scope_id.0),
            ScopeName::Property(name, scope_id) => write!(f, "{}#{}", name, scope_id.0),
        }
    }
}

/// A scope in a scope tree.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Scope {
    id: ScopeId,
    parent: Option<ScopeId>,
    children: Vec<ScopeId>,
    variables: HashMap<String, ScopeItem>,
    properties: HashMap<String, ScopeItem>,
}
impl Scope {
    pub fn new(id: ScopeId, parent: Option<ScopeId>) -> Self {
        Self {
            id,
            parent,
            children: vec![],
            variables: HashMap::new(),
            properties: HashMap::new(),
        }
    }

    pub fn id(&self) -> ScopeId {
        self.id
    }

    #[allow(dead_code)]
    pub fn parent(&self) -> Option<ScopeId> {
        self.parent
    }

    #[allow(dead_code)]
    pub fn children(&self) -> &Vec<ScopeId> {
        &self.children
    }

    pub fn get(&self, name: &ScopeName) -> Option<&ScopeItem> {
        match name {
            ScopeName::Variable(name, _) => self.variables.get(name),
            ScopeName::Property(name, _) => self.properties.get(name),
        }
    }

    pub fn get_property(&self, name: &str) -> Option<&PropertyValue> {
        self.properties
            .get(name)
            .and_then(|item| item.value.as_ref())
    }

    pub fn variables(&self) -> impl Iterator<Item = (&String, &UnresolvedPropertyValue)> {
        self.variables
            .iter()
            .map(|(name, item)| (name, &item.unresolved))
    }

    pub fn property_names(&self) -> impl Iterator<Item = &String> {
        self.properties
            .iter()
            .map(|(name, _)| name)
    }

    pub fn items(&self) -> impl Iterator<Item=(ScopeName, &ScopeItem)> {
        let variables = self.variables.iter()
            .map(|(name, entry)| (ScopeName::Variable(name.clone(), self.id), entry));
        let properties = self.properties.iter()
            .map(|(name, entry)| (ScopeName::Property(name.clone(), self.id), entry));
        
        variables.chain(properties)
    }

    #[allow(dead_code)]
    pub fn has_properties(&self) -> bool {
        !self.properties.is_empty()
    }

    pub fn add_variables<'a, I>(&mut self, variables: I)
    where I: IntoIterator<Item = (&'a String, &'a UnresolvedPropertyValue)> {
        for (name, value) in variables {
            self.variables.insert(
                name.clone(),
                ScopeItem {
                    unresolved: value.clone(),
                    value: None,
                },
            );
        }
    }

    pub fn add_resolved_variables<'a, I>(&mut self, variables: I)
    where I: IntoIterator<Item = (&'a String, &'a PropertyValue)> {
        for (name, value) in variables {
            self.variables.insert(
                name.clone(),
                ScopeItem {
                    unresolved: UnresolvedPropertyValue::Constant(value.clone()),
                    value: Some(value.clone()),
                },
            );
        }
    }

    pub fn add_properties<'a, I>(&mut self, properties: I)
    where I: IntoIterator<Item = (&'a String, &'a UnresolvedPropertyValue)> {
        for (name, value) in properties {
            self.properties.insert(
                name.clone(),
                ScopeItem {
                    unresolved: value.clone(),
                    value: None,
                },
            );
        }
    }

    pub fn merge(&mut self, other: &Scope) {
        self.add_properties(other.properties.iter().map(|(name, item)| (name, &item.unresolved)));
        self.add_variables(other.variables.iter().map(|(name, item)| (name, &item.unresolved)));
        self.children.extend(other.children.iter().cloned());
    }
}


lazy_static! {
    pub(crate) static ref EMPTY_SET: HashSet<ScopeName> = HashSet::new();
}

/// A dependency graph for scope names.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct DependencyGraph {
    /// A map for defining definition dependencies between scope names.
    /// Maps a scope name to its evaluation dependencies.
    map: HashMap<ScopeName, HashSet<ScopeName>>,
    /// A map for defining usage dependencies between scope names.
    /// Maps a scope name to other scope names that depend on it for evaluation.
    reverse_map: HashMap<ScopeName, HashSet<ScopeName>>,
    /// The topologically sorted order for the scope names in this dependency graph.
    order_list: Option<Vec<ScopeName>>,
    /// A map for easily getting the order of scope names.
    order_map: Option<HashMap<ScopeName, usize>>,
}
impl DependencyGraph {
    /// Adds a scope name to this dependency graph.
    pub fn add_node(&mut self, name: ScopeName) {
        self.map.entry(name).or_default();
    }

    /// Adds a dependency relation to the graph.
    pub fn add_dependency(&mut self, name: ScopeName, dependency: ScopeName) {
        let d = self.map.entry(name.clone()).or_default();
        d.insert(dependency.clone());
        let d = self.reverse_map.entry(dependency).or_default();
        d.insert(name);
    }

    /// Returns the scope names that depend on `name`.
    pub fn get_dependents(&self, name: &ScopeName) -> &HashSet<ScopeName> {
        self.reverse_map.get(name).unwrap_or(&EMPTY_SET)
    }

    /// Iterates over the nodes defined in this dependency graph.
    pub fn nodes(&self) -> impl Iterator<Item = &ScopeName> {
        self.map.iter().map(|(key, _)| key)
    }

    /// Returns the topologically sorted order of the nodes.
    #[allow(dead_code)]
    pub fn order(&self) -> &Vec<ScopeName> {
        self.order_list.as_ref().unwrap()
    }

    /// Returns a map for determining the index of each node
    /// in the topologically sorted order.
    pub fn order_map(&self) -> &HashMap<ScopeName, usize> {
        self.order_map.as_ref().unwrap()
    }

    /// Updates the topological sort for this graph.
    fn update_order(&mut self) {
        let mut visited: HashSet<&ScopeName> = HashSet::new();
        let mut path: Vec<&ScopeName> = Vec::new();
        let mut output: Vec<ScopeName> = Vec::new();

        fn dfs<'a>(
            node: &'a ScopeName,
            graph: &'a HashMap<ScopeName, HashSet<ScopeName>>,
            visited: &mut HashSet<&'a ScopeName>,
            path: &mut Vec<&'a ScopeName>,
            output: &mut Vec<ScopeName>,
        ) {
            if visited.contains(node) {
                return;
            }

            path.push(node);

            if let Some(deps) = graph.get(node) {
                for dep in deps {
                    if visited.contains(dep) {
                        continue;
                    }
                    if path.contains(&dep) {
                        let s = path
                            .iter()
                            .map(|l| format!("{}", l))
                            .collect::<Vec<_>>()
                            .join(", ");
                        panic!("cycle detected in dependency graph: {}", s);
                    }
                    dfs(dep, graph, visited, path, output);
                }
            }

            path.pop();
            visited.insert(node);
            output.push(node.clone());
        }

        for node in self.map.keys() {
            if !visited.contains(node) {
                dfs(&node, &self.map, &mut visited, &mut path, &mut output);
            }
        }

        let map = output
            .iter()
            .enumerate()
            .map(|(i, o)| (o.clone(), i))
            .collect::<HashMap<_, _>>();
        self.order_map = Some(map);
        self.order_list = Some(output);
    }

    /// Generates Graphviz' DOT code to visualize the dependency graph.
    #[allow(dead_code)]
    pub fn format_dot(&self) -> String {
        let mut out = String::new();

        writeln!(&mut out, "digraph DependencyGraph {{").unwrap();
        writeln!(&mut out, "  rankdir=TB;").unwrap();

        // Collect all nodes (keys + dependencies)
        let mut nodes = HashSet::new();
        for (name, deps) in &self.map {
            nodes.insert(name);
            for dep in deps {
                nodes.insert(dep);
            }
        }

        // Emit nodes
        for node in &nodes {
            writeln!(&mut out, r#"  "{}";"#, node).unwrap();
        }

        // Emit edges: name -> dependency
        for (name, deps) in &self.map {
            for dep in deps {
                writeln!(&mut out, r#"  "{}" -> "{}";"#, name, dep).unwrap();
            }
        }

        writeln!(&mut out, "}}").unwrap();
        out
    }
}

/// A structure for managing variables and
/// properties in the element hierarchy.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ScopeTree {
    /// The defined scopes.
    scopes: Vec<Scope>,
    dependency_graph: Option<DependencyGraph>,
}
impl ScopeTree {
    /// Creates a new scope.
    pub fn create(&mut self, parent: Option<ScopeId>) -> &mut Scope {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope::new(id.clone(), parent));

        if let Some(parent) = parent {
            if let Some(scope) = self.get_mut(parent) {
                scope.children.push(id);
            }
        }

        &mut self.scopes[id.0]
    }

    /// Returns a reference to scope with the given id.
    pub fn get(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(*id)
    }

    /// Returns a mutable reference to the scope with the given id.
    pub fn get_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(*id)
    }

    /// Returns a mutable reference for an item based on scope name.
    pub fn get_item_mut(&mut self, name: &ScopeName) -> Option<&mut ScopeItem> {
        let Some(scope) = self.get_mut(name.scope_id()) else {
            return None;
        };
        
        match name {
            ScopeName::Variable(name, _) => scope.variables.get_mut(name),
            ScopeName::Property(name, _) => scope.properties.get_mut(name),
        }
    }
    
    /// Returns a reference for an item based on scope name.
    pub fn get_entry(&self, name: &ScopeName) -> Option<&ScopeItem> {
        let Some(scope) = self.get(name.scope_id()) else {
            return None;
        };
        scope.get(name)
    }

    /// Finds the variable with `name` defined in the `start` scope or any of its parents
    /// in the hierarchy. Returns the variable item and the id of the scope that owns the variable,
    /// if any, otherwise returns `None`.
    pub fn find_variable(&self, name: &String, start: ScopeId) -> Option<(&ScopeItem, ScopeId)> {
        let mut scope = self.get(start)?;

        loop {
            if let Some(value) = scope.variables.get(name) {
                return Some((value, scope.id()));
            }
            scope = self.get(scope.parent?)?;
        }
    }

    /// Evaluates the scope name specified.
    pub fn evaluate(&mut self, name: &ScopeName) {
        let Some(item) = self.get_entry(name) else {
            return;
        };

        let value = match &item.unresolved {
            UnresolvedPropertyValue::Constant(value) => value.clone(),
            UnresolvedPropertyValue::Variable(variable) => {
                let value = self
                    .find_variable(variable, name.scope_id())
                    .and_then(|(item, _)| item.value.clone());
                match value {
                    Some(value) => value,
                    None => panic!("variable {name} not defined."),
                }
            }
        };

        let Some(item) = self.get_item_mut(name) else {
            return;
        };
        item.value = Some(value);
    }

    /// Updates the dependency graph of this scope tree.
    pub fn update_dependency_graph(&mut self) {
        let mut graph = DependencyGraph::default();

        // map to keep track of the variables in scope.
        let mut variables = HashMap::<String, ScopeId>::new();

        // perform a DFS in the tree
        let mut stack = vec![(ScopeId(0), false)];
        while let Some((id, post)) = stack.pop() {
            let Some(scope) = self.get(id) else { continue };

            // this means we already visited the children of this scope.
            // remove the variables of this scope from `variables`.
            if post {
                for name in scope.variables.keys() {
                    let Some(&origin_scope) = variables.get(name) else {
                        continue;
                    };
                    if origin_scope == id {
                        variables.remove(name);
                    }
                }
                continue;
            }

            // push this scope with post set to true to revisit it later
            stack.push((id, true));
            // push its children
            stack.extend(scope.children.iter().map(|c| (*c, false)).rev());

            variables.extend(scope.variables.iter().map(|(name, _)| (name.clone(), id)));

            for (name, entry) in scope.items() {
                graph.add_node(name.clone());

                match &entry.unresolved {
                    UnresolvedPropertyValue::Variable(variable) => {
                        let Some(&origin_scope) = variables.get(variable) else {
                            panic!("Undefined variable {}", variable);
                        };
                        graph.add_dependency(
                            name,
                            ScopeName::Variable(variable.clone(), origin_scope),
                        );
                    }
                    _ => {}
                }
            }
        }

        graph.update_order();
        self.dependency_graph = Some(graph);
    }

    /// Returns the dependency graph of this scope tree.
    pub fn dependency_graph(&self) -> &DependencyGraph {
        self.dependency_graph.as_ref().unwrap()
    }

    /// Generates Graphviz' DOT code to visualize the scope tree.
    #[allow(dead_code)]
    pub fn format_dot(&self) -> String {
        let mut out = String::new();

        writeln!(&mut out, "digraph ScopeTree {{").unwrap();
        writeln!(&mut out, "  rankdir=TB;").unwrap();
        writeln!(&mut out, "  node [shape=record, fontname=\"monospace\"];").unwrap();

        // Emit nodes
        for scope in &self.scopes {
            let mut label = String::new();

            // Header
            write!(&mut label, "#{}", scope.id.0).unwrap();

            // Variables (sorted for stable output)
            let mut vars: Vec<_> = scope.variables.iter().collect();
            vars.sort_by_key(|(k, _)| *k);

            for (name, entry) in vars {
                let value = format!("{}", entry.unresolved).replace("\"", "'");
                write!(&mut label, r"\n${}: {}", name, value).unwrap();
            }

            // Properties (sorted for stable output)
            let mut props: Vec<_> = scope.properties.iter().collect();
            props.sort_by_key(|(k, _)| *k);

            for (name, entry) in props {
                let value = format!("{}", entry.unresolved).replace("\"", "'");
                write!(&mut label, r"\n{}: {}", name, value).unwrap();
            }

            writeln!(&mut out, r#"  scope{} [label="{}"];"#, scope.id.0, label).unwrap();
        }

        // Emit edges
        for scope in &self.scopes {
            for &child in &scope.children {
                writeln!(&mut out, "  scope{} -> scope{};", scope.id.0, child.0).unwrap();
            }
        }

        writeln!(&mut out, "}}").unwrap();
        out
    }
}


lazy_static! {
    static ref EMPTY_ENTITY_SET: HashSet<Entity> = HashSet::new();
}

/// A structure for managing scope changes and triggering node updates.
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

    /// Removes a node entity from the list of listeners of the scope specified.
    pub fn remove(&mut self, scope: ScopeId, entity: Entity) {
        self.map.entry(scope).or_default().remove(&entity);
    }

    /// Returns an iterator of node entities that listen to changes in the given scope.
    pub fn get(&self, scope: ScopeId) -> impl Iterator<Item=Entity> {
        self.map.get(&scope).unwrap_or(&EMPTY_ENTITY_SET).iter().cloned()
    }
}
