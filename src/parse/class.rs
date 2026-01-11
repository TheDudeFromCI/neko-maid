//! Represents a hierarchy of classes applied to a widget for styling purposes.

use bevy::platform::collections::HashSet;

use crate::parse::context::{NekoResult, ParseContext};
use crate::parse::style::{Selector, SelectorPart};
use crate::parse::token::TokenType;

/// Represents a path of classes applied to a widget hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassPath {
    /// The hierarchy of classes in the class path.
    ///
    /// The first element is the root element in the hierarchy, with the final
    /// element being the current element.
    hierarchy: Vec<ClassSet>,
}

impl ClassPath {
    /// Creates a new [`ClassPath`] by stacking two existing class paths.
    pub fn stack(path1: &ClassPath, path2: &ClassPath) -> Self {
        let mut hierarchy = path1.hierarchy.clone();
        hierarchy.extend(path2.hierarchy.iter().cloned());
        Self { hierarchy }
    }

    /// Creates a new [`ClassPath`] with the given [`ClassSet`] as the root.
    pub fn new(classes: ClassSet) -> Self {
        Self {
            hierarchy: vec![classes],
        }
    }

    /// Appends another [`ClassSet`] to the end of this [`ClassPath`].
    pub fn append(&mut self, classes: ClassSet) {
        self.hierarchy.push(classes);
    }

    /// Checks if this [`ClassPath`] matches the given [`Selector`].
    pub fn matches(&self, selector: &Selector) -> bool {
        if self.hierarchy.len() < selector.hierarchy.len() {
            return false;
        }

        let offset = self.hierarchy.len() - selector.hierarchy.len();
        for depth in 0 .. selector.hierarchy.len() {
            let class_set = &self.hierarchy[depth + offset];
            let selector = &selector.hierarchy[depth];

            if !class_set.matches(selector) {
                return false;
            }
        }

        true
    }

    /// Checks if this [`ClassPath`] partially matches the given
    /// [`Selector`].
    pub fn partial_matches(&self, selector: &Selector) -> bool {
        if self.hierarchy.len() < selector.hierarchy.len() {
            return false;
        }

        let offset = self.hierarchy.len() - selector.hierarchy.len();
        for depth in 0 .. selector.hierarchy.len() {
            let class_set = &self.hierarchy[depth + offset];
            let selector = &selector.hierarchy[depth];

            if !class_set.partial_matches(selector) {
                return false;
            }
        }

        true
    }

    /// Returns a reference to the i-th [`ClassSet`] in relation to the path's
    /// end.
    pub fn get(&self, i: usize) -> Option<&ClassSet> {
        self.hierarchy.get(self.hierarchy.len() - i - 1)
    }

    /// Returns a mutable reference to the i-th [`ClassSet`] in relation to the
    /// path's end.
    pub fn get_mut(&mut self, i: usize) -> Option<&mut ClassSet> {
        let len = self.hierarchy.len();
        self.hierarchy.get_mut(len - i - 1)
    }

    /// Returns the last [`ClassSet`] in the class path.
    pub fn last(&self) -> &ClassSet {
        self.hierarchy.last().unwrap()
    }

    /// Returns a mutable reference to the last [`ClassSet`] in the class path.
    pub fn last_mut(&mut self) -> &mut ClassSet {
        self.hierarchy.last_mut().unwrap()
    }
}

/// Represents a set of classes applied to a widget.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassSet {
    /// The widget type.
    pub widget: String,

    /// The set of classes applied to the widget.
    pub classes: HashSet<String>,
}

impl ClassSet {
    /// Checks if this [`ClassSet`] matches the given [`SelectorPart`].
    pub fn matches(&self, selector: &SelectorPart) -> bool {
        if self.widget != selector.widget {
            return false;
        }

        for class in &selector.whitelist {
            if !self.classes.contains(class) {
                return false;
            }
        }

        for class in &selector.blacklist {
            if self.classes.contains(class) {
                return false;
            }
        }

        true
    }

    /// Checks if this [`ClassSet`] partially matches the given
    /// [`SelectorPart`].
    pub fn partial_matches(&self, selector: &SelectorPart) -> bool {
        self.widget == selector.widget
    }
}

/// Parses a class from the input and returns the class name as a string.
pub(super) fn parse_class(ctx: &mut ParseContext) -> NekoResult<String> {
    ctx.expect(TokenType::ClassKeyword)?;
    let class_name = ctx.expect_as_string(TokenType::Identifier)?;
    ctx.expect(TokenType::Semicolon)?;

    Ok(class_name)
}
