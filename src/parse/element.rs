//! A module for parsing and representing NekoMaid UI finalized elements.

use bevy::ecs::component::Component;
use bevy::platform::collections::{HashMap, HashSet};

use crate::parse::NekoMaidParseError;
use crate::parse::class::{ClassPath, ClassSet};
use crate::parse::context::NekoResult;
use crate::parse::layout::Layout;
use crate::parse::property::PropertyValue;
use crate::parse::style::Style;
use crate::parse::token::TokenPosition;
use crate::parse::widget::{NativeWidget, Widget, WidgetLayout};

/// A temporary builder for NekoMaid UI elements for easier construction.
#[derive(Debug, Clone, PartialEq)]
pub struct NekoElementBuilder {
    /// The native widget associated with this element.
    pub native_widget: NativeWidget,

    /// The NekoElement representing this element.
    pub element: NekoElement,

    /// The children of this element.
    pub children: Vec<NekoElementBuilder>,
}

/// A component representing a NekoMaid UI element.
#[derive(Debug, Component, Clone, PartialEq)]
pub struct NekoElement {
    /// The class path of this element.
    pub(crate) classpath: ClassPath,

    /// The styles applied to this element.
    styles: Vec<Style>,

    /// The properties applied to this element.
    properties: HashMap<String, PropertyValue>,
}

impl NekoElement {
    /// Creates a new NekoElement with the given class path and styles.
    pub fn new(classpath: ClassPath) -> Self {
        Self {
            classpath,
            styles: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Returns a reference to the class path of this element.
    pub fn classpath(&self) -> &ClassPath {
        &self.classpath
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
    pub fn try_add_style(&mut self, style: Style) {
        if self.classpath.partial_matches(style.selector()) {
            self.styles.insert(0, style);
        }
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

        None
    }
}

/// Builds a [`NekoElementBuilder`] from the given styles and layout.
pub fn build_element(
    global_variables: &HashMap<String, PropertyValue>,
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
                    global_variables,
                    styles,
                    widgets,
                    child,
                    Some(classpath.clone()),
                )?);
            }

            let mut element = NekoElement::new(classpath);

            for style in styles {
                element.try_add_style(style.clone());
            }

            for (name, value) in layout.properties {
                element.set_property(name, value);
            }

            Ok(NekoElementBuilder {
                element,
                children,
                native_widget,
            })
        }
        Widget::Custom(custom_widget) => {
            let mut custom_properties = global_variables.clone();
            for (name, value) in layout.properties {
                custom_properties.insert(name, value);
            }

            build_widget(
                global_variables,
                styles,
                widgets,
                custom_properties,
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
    global_variables: &HashMap<String, PropertyValue>,
    styles: &[Style],
    widgets: &HashMap<String, Widget>,
    custom_properties: HashMap<String, PropertyValue>,
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
                    global_variables,
                    styles,
                    widgets,
                    global_variables.clone(),
                    child,
                    original_children,
                    Some(classpath.clone()),
                )?);
            }

            if layout.is_output {
                for child in original_children.drain(..) {
                    children.push(build_element(
                        global_variables,
                        styles,
                        widgets,
                        child,
                        Some(classpath.clone()),
                    )?);
                }
            }

            let mut element = NekoElement::new(classpath);

            for style in styles {
                element.try_add_style(style.clone());
            }

            for (name, value) in layout.properties {
                let value = value.resolve(&custom_properties)?;
                element.set_property(name, value);
            }

            Ok(NekoElementBuilder {
                element,
                children,
                native_widget,
            })
        }
        Widget::Custom(custom_widget) => {
            let mut inner_custom_properties = global_variables.clone();
            for (name, value) in layout.properties {
                let value = value.resolve(&custom_properties)?;
                inner_custom_properties.insert(name, value);
            }

            build_widget(
                global_variables,
                styles,
                widgets,
                inner_custom_properties,
                custom_widget.layout,
                original_children,
                classpath,
            )
        }
    }
}
