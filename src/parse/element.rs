//! A module for parsing and representing NekoMaid UI finalized elements.

use std::sync::Arc;

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
use crate::parse::widget::{NativeWidget, Widget, WidgetLayout};

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

/// A component representing a NekoMaid UI element.
#[derive(Debug, Component, Clone, PartialEq)]
pub struct NekoElement {
    /// The class path of this element.
    classpath: ClassPath,

    /// The styles applied to this element.
    styles: Vec<Style>,

    /// The variables in scope for this element.
    variables: HashMap<String, UnresolvedPropertyValue>,

    /// The properties of this element before variable resolution.
    unresolved_properties: HashMap<String, UnresolvedPropertyValue>,

    /// The concrete properties applied to this element.
    properties: HashMap<String, PropertyValue>,

    /// The default properties of this element, from the native widget.
    default_properties: Arc<HashMap<String, PropertyValue>>,
}

impl NekoElement {
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
    /// Styles earlier in the vector have higher precedence.
    pub fn styles(&self) -> &Vec<Style> {
        &self.styles
    }

    /// Tries to add a style to the styles applied to this element. If the style
    /// has a selector that cannot match this element, it will not be added.
    pub fn try_add_style(&mut self, style: &Style) {
        if self.classpath.partial_matches(style.selector()) {
            self.styles.insert(0, style.clone());
        }
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
        if let Some(value) = self.properties.get(name) {
            return Some(value);
        };

        for style in &self.styles {
            if let Some(value) = style.get_property(name)
                && self.classpath.matches(style.selector())
            {
                return Some(value);
            }
        }

        self.default_properties.get(name)
    }

    /// Resolve properties for this element
    pub fn resolve(&mut self, variables: &HashMap<String, PropertyValue>) -> NekoResult<()> {
        for style in &mut self.styles {
            style.resolve(variables)?;
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

    /// Attempts to get a property and automatically convert it to the desired
    /// type. If the property is not found, returns the default value for the
    /// type.
    pub fn get_as<'a, O>(&'a self, name: &str) -> O
    where
        O: From<&'a PropertyValue> + Default,
    {
        self.get_property(name).map(Into::into).unwrap_or_default()
    }

    /// Attempts to get a property and automatically convert it to the desired
    /// type. If the property is not found, returns the provided default value.
    pub fn get_as_or<'a, O>(&'a self, name: &str, def: O) -> O
    where
        O: From<&'a PropertyValue>,
    {
        self.get_property(name).map(Into::into).unwrap_or(def)
    }

    /// Attempts to get a property, ignoring all default values provided by the
    /// native widget, and automatically convert it to the desired type. If the
    /// property is not found, returns the provided value.
    pub fn get_no_default<'a, O>(&'a self, name: &str, def: O) -> O
    where
        O: From<&'a PropertyValue>,
    {
        if let Some(value) = self.properties.get(name) {
            return value.into();
        };

        for style in &self.styles {
            if let Some(value) = style.get_property(name)
                && self.classpath.matches(style.selector())
            {
                return value.into();
            }
        }

        def
    }
}

/// Builds a [`NekoElementBuilder`] from the given styles and layout.
pub(super) fn build_element(
    styles: &[Style],
    widgets: &HashMap<String, Widget>,
    mut layout: Layout,
    classpath: Option<ClassPath>,
) -> NekoResult<NekoElementBuilder> {
    let Some(widget) = widgets.get(&layout.widget).cloned() else {
        return Err(NekoMaidParseError::UnknownWidget {
            widget: layout.widget,
            position: TokenPosition::default(),
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
            for child in layout.children {
                children.push(build_element(
                    styles,
                    widgets,
                    child,
                    Some(classpath.clone()),
                )?);
            }

            let mut element = NekoElement {
                classpath,
                styles: Vec::new(),
                variables: HashMap::new(),
                unresolved_properties: layout.properties,
                properties: HashMap::new(),
                default_properties: native_widget.default_properties.clone(),
            };

            for style in styles {
                element.try_add_style(style);
            }

            Ok(NekoElementBuilder {
                element,
                children,
                native_widget,
            })
        }
        Widget::Custom(custom_widget) => {
            let mut local_variables = custom_widget.default_properties.clone();

            for (name, value) in layout.properties {
                local_variables.insert(name, value);
            }

            build_widget(
                &local_variables,
                styles,
                widgets,
                custom_widget.layout,
                &mut layout.children,
                classpath,
            )
        }
    }
}

/// Builds a [`NekoElementBuilder`] from the given styles and custom widget
/// layout.
fn build_widget(
    variables: &HashMap<String, UnresolvedPropertyValue>,
    styles: &[Style],
    widgets: &HashMap<String, Widget>,
    layout: WidgetLayout,
    original_children: &mut Vec<Layout>,
    classpath: Option<ClassPath>,
) -> NekoResult<NekoElementBuilder> {
    let Some(widget) = widgets.get(&layout.widget).cloned() else {
        return Err(NekoMaidParseError::UnknownWidget {
            widget: layout.widget,
            position: TokenPosition::default(),
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
            for child in layout.children {
                children.push(build_widget(
                    &variables,
                    styles,
                    widgets,
                    child,
                    original_children,
                    Some(classpath.clone()),
                )?);
            }

            if layout.is_output {
                for child in original_children.drain(..) {
                    children.push(build_element(
                        styles,
                        widgets,
                        child,
                        Some(classpath.clone()),
                    )?);
                }
            }

            let mut element = NekoElement {
                classpath,
                styles: Vec::new(),
                variables: variables.clone(),
                unresolved_properties: HashMap::new(),
                properties: HashMap::new(),
                default_properties: native_widget.default_properties.clone(),
            };

            for style in styles {
                element.try_add_style(style);
            }

            for (name, value) in layout.properties {
                element
                    .unresolved_properties
                    .insert(name.clone(), value.clone());
            }

            Ok(NekoElementBuilder {
                element,
                children,
                native_widget,
            })
        }
        Widget::Custom(custom_widget) => {
            let mut local_variables = variables.clone();

            for (name, value) in custom_widget.default_properties {
                local_variables.insert(name, value);
            }

            for (name, value) in layout.properties {
                local_variables.insert(name, value);
            }

            build_widget(
                &local_variables,
                styles,
                widgets,
                custom_widget.layout,
                original_children,
                classpath,
            )
        }
    }
}
