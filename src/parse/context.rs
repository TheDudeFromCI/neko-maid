//! Temporary context for parsing NekoMaid UI files.

use std::iter::Peekable;
use std::vec::IntoIter;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::parse::NekoMaidParseError;
use crate::parse::element::{NekoElementBuilder, build_tree};
use crate::parse::layout::Layout;
use crate::parse::module::Module;
use crate::parse::property::UnresolvedPropertyValue;
use crate::parse::scope::{Scope, ScopeId, ScopeTree};
use crate::parse::style::Style;
use crate::parse::token::{Token, TokenPosition, TokenType, TokenValue};
use crate::parse::widget::Widget;

/// Context for parsing NekoMaid UI files.
pub(crate) struct ParseContext {
    /// The scope tree for this parse context.
    scope_tree: ScopeTree,

    /// A list of defined styles.
    styles: Vec<Style>,

    /// A list of defined layouts.
    layouts: Vec<Layout>,

    /// A map of available widgets.
    widgets: HashMap<String, Widget>,

    /// A list of modules that can be imported.
    modules: HashMap<String, Module>,

    /// The tokens being parsed.
    tokens: Peekable<IntoIter<Token>>,

    /// A list of elements imported from other modules.
    imported_elements: Vec<NekoElementBuilder>,

    /// the name of the widget currently being parsed.
    current_widget: Option<String>,
}

impl ParseContext {
    /// Creates a new, empty [`ParseContext`].
    ///
    /// A file retriever function can be provided to enable importing of
    /// external NekoMaid UI modules.
    pub(crate) fn new(tokens: Vec<Token>) -> Self {
        // create global scope
        let mut scope = ScopeTree::default();
        scope.create(None);

        Self {
            scope_tree: scope,
            styles: Vec::new(),
            layouts: Vec::new(),
            widgets: HashMap::new(),
            modules: HashMap::new(),
            tokens: tokens.into_iter().peekable(),
            imported_elements: Vec::new(),
            current_widget: None,
        }
    }

    /// Peeks at the next token without advancing the index.
    pub(crate) fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    /// Advances to the next token and returns it.
    pub(crate) fn consume(&mut self) -> Result<Token, NekoMaidParseError> {
        self.tokens.next().ok_or(NekoMaidParseError::EndOfStream)
    }

    /// Checks if the next token matches the given type and advances if it does,
    /// returning the token's value.
    pub(super) fn maybe_consume(&mut self, test: TokenType) -> Option<Token> {
        let next = self.tokens.peek()?;
        if next.token_type == test {
            Some(self.tokens.next().unwrap())
        } else {
            None
        }
    }

    /// Expects the next token to be of the given type, advancing the index and
    /// returning the token's value. Returns an error if the next token does not
    /// match the expected type.
    pub(super) fn expect(&mut self, expected: TokenType) -> Result<Token, NekoMaidParseError> {
        let next = self.consume()?;

        if next.token_type == expected {
            Ok(next)
        } else {
            Err(NekoMaidParseError::UnexpectedToken {
                expected: vec![expected.type_name().to_string()],
                found: next.token_type.type_name().to_string(),
                position: next.position,
            })
        }
    }

    /// Expects the next token to be of the given type, advancing the index and
    /// returning the token's value as a string. Returns an error if the next
    /// token does not match the expected type or cannot be converted to a
    /// string.
    pub(crate) fn expect_as_string(
        &mut self,
        expected: TokenType,
    ) -> Result<String, NekoMaidParseError> {
        let next_pos = self.next_position().unwrap_or_default();
        let next = self.consume()?;

        if next.token_type == expected {
            match next.value {
                TokenValue::String(s) => Ok(s),
                _ => Err(NekoMaidParseError::InvalidTokenValue {
                    expected: "string".to_string(),
                    found: format!("{:?}", next.value),
                    position: next_pos,
                }),
            }
        } else {
            Err(NekoMaidParseError::UnexpectedToken {
                expected: vec![expected.type_name().to_string()],
                found: next.token_type.type_name().to_string(),
                position: next.position,
            })
        }
    }

    /// Sets the value of a defined variable. If the variable already exists,
    /// its value is updated.
    pub(crate) fn set_variable(&mut self, name: &String, value: &UnresolvedPropertyValue) {
        let Some(scope) = self.scope_tree.get_mut(ScopeId(0)) else {
            return;
        };
        scope.add_variables([(name, value)]);
    }

    /// Creates and returns a scope that is child of the provided scope.
    pub(crate) fn create_scope(&mut self, parent: ScopeId) -> &mut Scope {
        self.scope_tree.create(Some(parent))
    }

    /// Converts this parse context into a [`Module`].
    pub(crate) fn into_module(self) -> NekoResult<Module> {
        let mut elements = self.imported_elements;

        let global_scope_id = ScopeId(0);
        let mut scope_tree = self.scope_tree;

        for layout in self.layouts {
            let element = build_tree(
                global_scope_id,
                &mut scope_tree,
                &self.styles,
                &self.widgets,
                layout,
            )?;
            elements.push(element);
        }

        scope_tree.update_dependency_graph();

        Ok(Module {
            scope: scope_tree,
            styles: self.styles,
            widgets: self.widgets,
            elements,
        })
    }

    /// Gets the next token position in the token stream, or `None` if there are
    /// no more tokens.
    pub(crate) fn next_position(&mut self) -> Option<TokenPosition> {
        self.peek().map(|t| t.position)
    }

    /// Adds a widget definition to the list of available widgets.
    pub(crate) fn add_widget(&mut self, widget: Widget) {
        self.widgets.insert(widget.name().to_string(), widget);
    }

    /// Gets the widget definition for the given widget name, if it exists.
    pub(crate) fn get_widget(&self, widget: &str) -> Option<&Widget> {
        self.widgets.get(widget)
    }

    /// Adds a style definition to the list of styles. If two styles have equal
    /// selectors, they will be merged together. In the case of property
    /// conflicts, the properties of the later-added style will take
    /// precedence.
    pub(crate) fn add_style(&mut self, style: Style) {
        for existing_style in &mut self.styles {
            if existing_style.selector() == style.selector() {
                let Some(scope) = self.scope_tree.get(style.scope_id).cloned() else {
                    return;
                };
                let Some(existing_scope) = self.scope_tree.get_mut(style.scope_id) else {
                    return;
                };
                existing_scope.merge(&scope);
                return;
            }
        }

        self.styles.push(style);
    }

    /// Adds a layout to the list of elements.
    pub(crate) fn add_layout(&mut self, layout: Layout) {
        self.layouts.push(layout);
    }

    /// Attempts to import a module by its name. The module must have been
    /// previously added to this context via [`add_module`].
    ///
    /// Importing a module will destroy temporary metadata associated with it,
    /// and prevent it from being imported again.
    pub(crate) fn import_module(
        &mut self,
        name: &str,
        pos: TokenPosition,
    ) -> Result<(), NekoMaidParseError> {
        let Some(module) = self.modules.remove(name) else {
            return Err(NekoMaidParseError::ModuleNotFound {
                name: name.to_string(),
                position: pos,
            });
        };

        if let Some(global_scope) = module.scope.get(ScopeId(0)) {
            for (var_name, var_value) in global_scope.variables() {
                self.set_variable(var_name, var_value);
            }
        }

        for style in module.styles {
            self.add_style(style);
        }

        self.imported_elements.extend(module.elements);

        for (_, widget) in module.widgets {
            self.add_widget(widget);
        }

        Ok(())
    }

    /// Adds a module to this context under the given name.
    ///
    /// This does not import the module; it simply makes it available for import
    /// within this context if requested.
    pub(crate) fn add_module(&mut self, name: String, module: Module) {
        self.modules.insert(name, module);
    }

    /// Gets the name of the widget currently being parsed.
    pub(super) fn get_current_widget(&self) -> &Option<String> {
        &self.current_widget
    }

    /// Sets the name of the widget currently being parsed.
    pub(super) fn set_current_widget(&mut self, name: Option<String>) {
        self.current_widget = name;
    }
}

/// A specialized result type for NekoMaid parsing operations.
pub type NekoResult<T> = Result<T, NekoMaidParseError>;
