//! A module for parsing and representing NekoMaid UI finalized elements.

use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{Deref, DerefMut};

use crate::parse::NekoMaidParseError;
use crate::parse::class::{ClassPath, ClassSet};
use crate::parse::context::NekoResult;
use crate::parse::layout::Layout;
use crate::parse::scope::{ScopeId, ScopeTree};
use crate::parse::style::Style;
use crate::parse::token::TokenPosition;
use crate::parse::value::PropertyValue;
use crate::parse::widget::{NativeWidget, Widget};

/// A temporary builder for NekoMaid UI elements for easier construction.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NekoElementBuilder {
    /// The native widget associated with this element.
    pub(crate) native_widget: NativeWidget,

    /// The NekoElement representing this element.
    pub(crate) element: NekoElement,

    /// The children of this element.
    pub(crate) children: Vec<NekoElementBuilder>,
}

/// A style entry in an element.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StyleEntry {
    /// The style.
    pub value: Style,
    /// Whether the current style is active i.e matches the current class path.
    pub active: bool,
}

/// A NekoMaid UI element.
#[derive(Debug, Clone, PartialEq)]
pub struct NekoElement {
    /// The class path of this element.
    classpath: ClassPath,
    pub(crate) classpath_changed: bool,
    pub(crate) added_classes: Vec<String>,
    pub(crate) removed_classes: Vec<String>,

    /// The styles applied to this element.
    pub(crate) styles: Vec<StyleEntry>,
    pub(crate) activated_styles: Vec<usize>,
    pub(crate) deactivated_styles: Vec<usize>,

    /// A map that tells where a property applied to this
    /// element comes from. If `Some(i)`, the property
    /// comes from the i-th style, while if it's `None`,
    /// the property is local to this element and lives
    /// in the element scope.
    active_properties: HashMap<String, Option<usize>>,
    dirty_active_properties: bool,

    /// Scope id
    scope: ScopeId,
}

impl NekoElement {
    /// Creates a new element.
    pub(crate) fn new(classpath: ClassPath, scope_id: ScopeId) -> Self {
        Self {
            classpath,
            classpath_changed: true,
            added_classes: Vec::new(),
            removed_classes: Vec::new(),
            styles: Vec::new(),
            activated_styles: Vec::new(),
            deactivated_styles: Vec::new(),
            active_properties: HashMap::new(),
            dirty_active_properties: false,
            scope: scope_id,
        }
    }

    /// Returns a reference to the class path of this element.
    pub fn classpath(&self) -> &ClassPath {
        &self.classpath
    }

    /// Returns a mutable reference to the class path of this element.
    pub fn classpath_mut(&mut self) -> &mut ClassPath {
        self.classpath_changed = true;
        &mut self.classpath
    }

    /// Returns a reference to the set of classes applied to this element.
    pub fn classes(&self) -> &HashSet<String> {
        &self.classpath.last().classes
    }

    /// Adds a class to the class path of this element.
    pub fn add_class(&mut self, class: String) {
        if self.classpath.last_mut().classes.insert(class.clone()) {
            self.classpath_changed = true;
            self.added_classes.push(class)
        }
        
    }

    /// Removes a class from the class path of this element.
    pub fn remove_class(&mut self, class: &str) {
        if self.classpath.last_mut().classes.remove(class) {
            self.classpath_changed = true;
            self.removed_classes.push(class.to_string())
        }
    }

    /// Updates the list of active styles.
    pub fn update_active_styles(&mut self) {
        for (i, style) in self.styles.iter_mut().enumerate() {
            let active = self.classpath.matches(style.value.selector());

            if style.active != active {
                style.active = active;
                self.dirty_active_properties = true;

                if active {
                    self.activated_styles.push(i);
                }
                else {
                    self.deactivated_styles.push(i);
                }
            }
        }
        self.classpath_changed = false;
    }

    /// Returns a reference to the styles applied to this element.
    ///
    /// Styles earlier in the vector have lower precedence.
    pub fn active_styles(&self) -> impl Iterator<Item = &Style> {
        self.styles.iter()
            .filter(|e| e.active)
            .map(|e| &e.value)
    }

    /// Tries to add a style to the styles applied to this element. If the style
    /// has a selector that cannot match this element, it will not be added.
    pub fn try_add_style(&mut self, style: &Style) {
        if self.classpath.partial_matches(style.selector()) {
            let active = self.classpath.matches(style.selector());

            let entry = StyleEntry {
                value: style.clone(),
                active,
            };
            let i = self.styles.len();
            self.styles.push(entry);

            if active {
                self.dirty_active_properties = true;
                self.activated_styles.push(i);
            }
        }
    }

    /// Returns the name of all active properties in this element,
    /// including indirect properties coming from styles.
    pub fn active_properties(&self) -> impl Iterator<Item = &String> {
        self.active_properties.keys()
    }

    /// Returns the id of the scope used by this element.
    pub(crate) fn scope_id(&self) -> ScopeId {
        self.scope
    }

    /// Returns a mutable view on the element's properties given scope context.
    pub(crate) fn view_mut<'a>(&'a mut self, scopes: &'a mut ScopeTree) -> NekoElementView<'a> {
        NekoElementView { el: self, scopes }
    }
}

/// A view on the element's properties given scope context.
#[derive(Debug, Deref, DerefMut)]
pub struct NekoElementView<'a> {
    #[deref]
    el: &'a mut NekoElement,
    scopes: &'a mut ScopeTree,
}

impl<'a> NekoElementView<'a> {
    /// Updates the list of all properties applied to this element.
    pub fn update_active_properties(&mut self) {
        if self.classpath_changed {
            self.update_active_styles();
        }

        self.active_properties.clear();

        let Some(scope) = self.scopes.get(self.scope) else {
            return;
        };
        for name in scope.property_names() {
            self.el.active_properties.insert(name.clone(), None);
        }

        for i in (0 .. self.styles.len()).rev() {
            if !self.styles[i].active {
                continue;
            }
            self.update_style_properties(i);
        }

        self.dirty_active_properties = false;
    }
    fn update_style_properties(&mut self, i: usize) {
        let style = &self.styles[i].value;
        let Some(scope) = self.scopes.get(style.scope_id) else { return };
        for name in scope.property_names() {
            let j = match self.active_properties.get(name) {
                Some(j) => j.unwrap_or(usize::MAX),
                None => 0,
            };
            if i >= j {
                self.el.active_properties.insert(name.clone(), Some(i));
            }
        }
    }

    /// Gets a property defined by the current style of this element.
    #[inline(always)]
    pub fn get_property(&mut self, name: &str) -> Option<&PropertyValue> {
        if self.dirty_active_properties {
            self.update_active_properties();
        }

        let origin = self.active_properties.get(name)?;
        match *origin {
            Some(i) => {
                let style = &self.styles[i].value;
                let scope = self.scopes.get(style.scope_id)?;
                scope.get_property(name)
            }
            None => {
                let scope = self.scopes.get(self.scope)?;
                scope.get_property(name)
            }
        }
    }

    /// Attempts to get a property and automatically convert it to the desired
    /// type. If the property is not found, returns the default value for the
    /// type.
    #[inline(always)]
    pub fn get_as<'b, O>(&'b mut self, name: &str) -> Option<O>
    where
        O: From<&'b PropertyValue> + Default,
    {
        self.get_property(name).map(Into::into)
    }

    /// Attempts to get a property and automatically convert it to the desired
    /// type. If the property is not found, returns the provided default value.
    #[inline(always)]
    pub fn get_as_or<'b, O>(&'b mut self, name: &str, def: O) -> O
    where
        O: From<&'b PropertyValue>,
    {
        self.get_property(name).map(Into::into).unwrap_or(def)
    }
}

/// Builds an element tree.
pub(super) fn build_tree(
    global_scope: ScopeId,
    scopes: &mut ScopeTree,
    styles: &[Style],
    widgets: &HashMap<String, Widget>,
    layout: Layout,
) -> NekoResult<NekoElementBuilder> {
    build_element(global_scope, scopes, styles, widgets, layout, None)
}

/// Builds a [`NekoElementBuilder`] from the given styles and layout.
pub(super) fn build_element(
    parent_scope: ScopeId,
    scopes: &mut ScopeTree,
    styles: &[Style],
    widgets: &HashMap<String, Widget>,
    layout: Layout,
    classpath: Option<ClassPath>,
) -> NekoResult<NekoElementBuilder> {
    let Some(widget) = widgets.get(&layout.widget) else {
        return Err(NekoMaidParseError::UnknownWidget {
            widget: layout.widget.clone(),
            position: TokenPosition::UNKNOWN,
        });
    };

    match widget {
        Widget::Native(native_widget) => {
            let classes = ClassSet {
                widget: layout.widget,
                classes: HashSet::new(),
            };
            let classpath = match classpath {
                Some(mut path) => {
                    path.append(classes);
                    path
                }
                None => ClassPath::new(classes),
            };

            let scope = scopes.create(Some(parent_scope));
            scope.add_properties(layout.properties.iter());
            let scope_id = scope.id();

            let mut element = NekoElement::new(classpath, scope_id);
            for class in layout.classes {
                element.add_class(class);
            }
            for style in styles {
                element.try_add_style(style);
            }
            element.view_mut(scopes).update_active_properties();

            let mut children = Vec::new();
            if let Some(c) = layout.children_slots.get("default") {
                for child in c {
                    children.push(build_element(
                        scope_id,
                        scopes,
                        styles,
                        widgets,
                        child.clone(),
                        Some(element.classpath().clone()),
                    )?);
                }
            }

            Ok(NekoElementBuilder {
                element,
                children,
                native_widget: native_widget.clone(),
            })
        }
        Widget::Custom(custom_widget) => {
            let widget_scope = scopes.create(Some(parent_scope));
            widget_scope.add_variables(custom_widget.default_properties.iter());
            widget_scope.add_variables(layout.properties.iter());

            let mut widget_layout = custom_widget.layout.clone();
            substitute_widget_slots(&mut widget_layout, layout.children_slots);

            build_element(
                widget_scope.id(),
                scopes,
                styles,
                widgets,
                widget_layout,
                classpath,
            )
        }
    }
}

/// Insert the given nodes into the slots of this layout hierarchy.
pub(super) fn substitute_widget_slots(
    layout: &mut Layout,
    mut slots: HashMap<String, Vec<Layout>>,
) -> HashMap<String, Vec<Layout>> {
    // the slot list is sorted in ascending order by index position.
    // it's important to substitute the slots in the end first to
    // not mess up the indices when inserting elements to the children vector.
    //
    // by popping the slot from this layout we guarantee that it's not mistakenly
    // used twice.
    while let Some(slot) = layout.slots.pop() {
        let layout_children = layout.get_slot_mut(slot.location);

        if let Some(mut children) = slots.remove(&slot.name) {
            // we should insert in reverse order since we always
            // insert at the beginning
            children.reverse();
            for mut c in children {
                // guarantee that the slot content does not have any remaining slots
                c.slots.clear();
                layout_children.insert(slot.index, c);
            }
        }
    }

    for children in layout.children_slots.values_mut() {
        for c in children {
            slots = substitute_widget_slots(c, slots);
        }
    }
    slots
}
