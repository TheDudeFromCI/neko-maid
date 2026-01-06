//! A module for parsing and representing NekoMaid UI finalized elements.

use bevy::ecs::component::Component;
use bevy::platform::collections::{HashMap, HashSet};

use crate::parse::NekoMaidParseError;
use crate::parse::class::{ClassPath, ClassSet};
use crate::parse::context::NekoResult;
use crate::parse::layout::Layout;
use crate::parse::property::UnresolvedPropertyValue;
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
pub struct StyleEntry {
    /// The style.
    value: Style,
    /// Whether the current style is active i.e matches the current class path.
    active: bool,
}

/// A component representing a NekoMaid UI element.
#[derive(Debug, Component, Clone, PartialEq)]
pub struct NekoElement {
    /// The class path of this element.
    classpath: ClassPath,

    /// The styles applied to this element.
    styles: Vec<StyleEntry>,

    /// A map that tells where a property applied to this
    /// element comes from. If `Some(i)`, the property
    /// comes from the i-th style, while if it's `None`,
    /// the property is local to this element and lives
    /// in the `properties` map.
    active_properties: HashMap<String, Option<usize>>,

    /// The variables in scope for this element.
    variables: HashMap<String, UnresolvedPropertyValue>,

    /// The properties of this element before variable resolution.
    unresolved_properties: HashMap<String, UnresolvedPropertyValue>,

    /// The concrete properties applied to this element.
    properties: HashMap<String, PropertyValue>,
}

impl NekoElement {
    /// Creates a new element.
    pub(crate) fn new(
        classpath: ClassPath,
        variables: HashMap<String, UnresolvedPropertyValue>,
        unresolved_properties: HashMap<String, UnresolvedPropertyValue>,
    ) -> Self {
        let mut s = Self {
            classpath,
            styles: Vec::new(),
            active_properties: HashMap::new(),
            variables,
            unresolved_properties,
            properties: HashMap::new(),
        };
        s.update_active_properties();
        s
    }

    /// Returns a reference to the class path of this element.
    pub fn classpath(&self) -> &ClassPath {
        &self.classpath
    }

    /// Returns a mutable reference to the class path of this element.
    pub fn classpath_mut(&mut self) -> &mut ClassPath {
        &mut self.classpath
    }

    /// Returns a reference to the set of classes applied to this element.
    pub fn classes(&self) -> &HashSet<String> {
        &self.classpath.last().classes
    }

    /// Adds a class to the class path of this element.
    pub fn add_class(&mut self, class: String) {
        if self.classpath.last_mut().classes.insert(class) {}
    }

    /// Removes a class from the class path of this element.
    pub fn remove_class(&mut self, class: &str) {
        self.classpath.last_mut().classes.remove(class);
    }

    /// Returns a reference to the styles applied to this element.
    ///
    /// Styles earlier in the vector have lower precedence.
    pub fn styles(&self) -> impl Iterator<Item = &Style> {
        self.styles.iter().map(|e| &e.value)
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
            self.styles.push(entry);

            if active {
                self.update_style_properties(self.styles.len() - 1);
            }
        }
    }

    /// Updates the list of all properties applied to this element.
    pub fn update_active_properties(&mut self) {
        self.active_properties.clear();

        for (name, _) in &self.unresolved_properties {
            self.active_properties.insert(name.clone(), None);
        }

        for i in (0 .. self.styles.len()).rev() {
            if !self.styles[i].active {
                continue;
            }
            self.update_style_properties(i);
        }
    }
    fn update_style_properties(&mut self, i: usize) {
        let style_properties = self.styles[i].value.unresolved_properties.keys();
        for name in style_properties {
            let j = match self.active_properties.get(name) {
                Some(j) => j.unwrap_or(usize::MAX),
                None => 0,
            };
            if i >= j {
                self.active_properties.insert(name.clone(), Some(i));
            }
        }
    }
    /// Returns the name of all active properties in this element,
    /// including indirect properties coming from styles.
    pub fn active_properties(&self) -> impl Iterator<Item = &String> {
        self.active_properties.keys()
    }

    /// Resolve properties for this element
    pub fn resolve(&mut self, variables: &HashMap<String, PropertyValue>) -> NekoResult<()> {
        for style in &mut self.styles {
            style.value.resolve(variables)?;
        }

        let mut variables = variables.clone();
        for (name, value) in &self.variables {
            let prop = value.resolve(&variables)?;
            variables.insert(name.clone(), prop);
        }

        for (name, value) in &self.unresolved_properties {
            let prop = value.resolve(&variables)?;
            self.properties.insert(name.clone(), prop);
        }

        Ok(())
    }

    /// Returns a reference to the property map of this element.
    pub fn properties(&self) -> &HashMap<String, PropertyValue> {
        &self.properties
    }

    /// Sets a property directly on this element, overriding all styles.
    pub fn set_property(&mut self, name: String, value: PropertyValue) {
        self.properties.insert(name, value);
    }

    /// Gets a property defined by the current style of this element.
    ///
    /// Note that this may be slow if there are many styles applied to this
    /// element. It is recommended to only check for properties when
    /// necessary (e.g. on class changes or style updates).
    pub fn get_property(&self, name: &str) -> Option<&PropertyValue> {
        let origin = self.active_properties.get(name)?;
        match *origin {
            Some(i) => self.styles[i].value.get_property(name),
            None => self.properties.get(name),
        }
    }

    /// Attempts to get a property and automatically convert it to the desired
    /// type. If the property is not found, returns the default value for the
    /// type.
    pub fn get_as<'a, O>(&'a self, name: &str) -> Option<O>
    where
        O: From<&'a PropertyValue> + Default,
    {
        self.get_property(name).map(Into::into)
    }

    /// Attempts to get a property and automatically convert it to the desired
    /// type. If the property is not found, returns the provided default value.
    pub fn get_as_or<'a, O>(&'a self, name: &str, def: O) -> O
    where
        O: From<&'a PropertyValue>,
    {
        self.get_property(name).map(Into::into).unwrap_or(def)
    }
}

/// Builds a [`NekoElementBuilder`] from the given styles and layout.
pub(super) fn build_element(
    variables: &HashMap<String, UnresolvedPropertyValue>,
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
                classes: layout.classes,
            };

            let classpath = match classpath {
                Some(mut path) => {
                    path.append(classes);
                    path
                }
                None => ClassPath::new(classes),
            };

            let mut children = Vec::new();
            if let Some(c) = layout.children_slots.get("default") {
                for child in c {
                    children.push(build_element(
                        variables,
                        styles,
                        widgets,
                        child.clone(),
                        Some(classpath.clone()),
                    )?);
                }
            }

            let mut element = NekoElement::new(classpath, variables.clone(), layout.properties);
            for style in styles {
                element.try_add_style(style);
            }

            Ok(NekoElementBuilder {
                element,
                children,
                native_widget: native_widget.clone(),
            })
        }
        Widget::Custom(custom_widget) => {
            let mut local_variables = variables.clone();

            for (name, value) in &custom_widget.default_properties {
                local_variables.insert(name.clone(), value.clone());
            }

            for (name, value) in &layout.properties {
                local_variables.insert(name.clone(), value.clone());
            }

            let mut widget_layout = custom_widget.layout.clone();
            substitute_widget_slots(&mut widget_layout, layout.children_slots);

            build_element(&local_variables, styles, widgets, widget_layout, classpath)
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
