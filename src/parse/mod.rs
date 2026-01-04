//! This module implements the parsing functionality for NekoMaid UI files.
//! It provides functions to read and interpret `.neko_ui` files.

use crate::parse::context::{NekoResult, ParseContext};
use crate::parse::import::predict_imports;
use crate::parse::module::Module;
use crate::parse::token::TokenPosition;
use crate::parse::tokenizer::{TokenizeError, Tokenizer};
use crate::parse::widget::{NativeWidget, Widget};

pub mod class;
pub mod context;
pub mod element;
pub mod import;
pub mod layout;
pub mod module;
pub mod property;
pub mod style;
pub mod token;
pub mod tokenizer;
pub mod value;
pub mod widget;

/// A parser for NekoMaid UI files.
pub struct NekoMaidParser {
    /// The parsing context.
    context: ParseContext,

    /// The predicted imports required by the tokens.
    imports: Vec<String>,
}

impl NekoMaidParser {
    /// Tokenizes the given NekoMaid UI code into a vector of tokens.
    pub fn tokenize(code: &str) -> NekoResult<Self> {
        let tokens = Tokenizer::tokenize(code)?;
        let imports = predict_imports(&tokens);

        Ok(Self {
            context: ParseContext::new(tokens),
            imports,
        })
    }

    /// Registers a native widget within this parser's context.
    pub fn register_native_widget(&mut self, widget: NativeWidget) {
        self.context.add_widget(Widget::Native(widget));
    }

    /// Predicts the imports required by the given tokens.
    ///
    /// This function is not guaranteed to be accurate if the tokens are
    /// malformed.
    pub fn predict_imports(&self) -> &Vec<String> {
        &self.imports
    }

    /// Adds a module to this parser's context under the given name.
    ///
    /// This does not import the module; it simply makes it available for import
    /// within this context if requested.
    pub fn add_module(&mut self, name: String, module: Module) {
        self.context.add_module(name, module);
    }

    /// Finishes parsing and returns the resulting module.
    pub fn finish(self) -> NekoResult<Module> {
        module::parse_module(self.context)
    }
}

/// Errors that can occur during parsing of NekoMaid UI files.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum NekoMaidParseError {
    /// Error during tokenization
    #[error("{0}")]
    TokenizerError(#[from] TokenizeError),

    /// Unexpected token encountered
    #[error("Unexpected token at {position}: found {found}, expected one of: {expected:?}")]
    UnexpectedToken {
        /// The expected token list.
        expected: Vec<String>,

        /// The found token description.
        found: String,

        /// The position of the unexpected token.
        position: TokenPosition,
    },

    /// Unexpected end of input
    #[error("Unexpected end of input")]
    EndOfStream,

    /// An error that occurs due to a token storing an invalid value for its
    /// type. This is an internal error and should not occur during normal
    /// parsing.
    #[error("Invalid token value: expected {expected}, found {found} at {position}")]
    InvalidTokenValue {
        /// The expected token value type.
        expected: String,

        /// The found token value type.
        found: String,

        /// The position of the invalid token.
        position: TokenPosition,
    },

    /// An error indicating that a variable could not be found.
    #[error("Variable not found: {variable}, at {position}")]
    VariableNotFound {
        /// The name of the variable that was not found.
        variable: String,

        /// The position where the variable was referenced.
        position: TokenPosition,
    },

    /// An error indicating that a widget definition is incomplete.
    #[error("Incomplete widget definition for '{widget}' at {position}, no layout defined")]
    IncompleteWidgetDefinition {
        /// The name of the widget with the incomplete definition.
        widget: String,

        /// The position of the widget definition in the source code.
        position: TokenPosition,
    },

    /// An error indicating that an unknown widget was referenced.
    #[error("Unknown widget '{widget}' at {position}")]
    UnknownWidget {
        /// The name of the unknown widget.
        widget: String,

        /// The position of the widget reference in the source code.
        position: TokenPosition,
    },

    /// An error indicating that a module could not be found.
    #[error("Module not found: {name}, at {position}")]
    ModuleNotFound {
        /// The name of the module that was not found.
        name: String,

        /// The position where the module was referenced.
        position: TokenPosition,
    },

    /// An error indicating that multiple layouts were defined in a single
    /// widget definition.
    #[error("A widget cannot have multiple layouts defined: {position}")]
    MultipleLayoutsDefined {
        /// The position of the second layout definition.
        position: TokenPosition,
    },

    /// An error indicating that a slot was provided twice
    /// in a layout.
    #[error("Slot {slot} was provided twice at {position}.")]
    InputSlotProvidedTwice {
        /// The name of the slot provided twice.
        slot: String,

        /// The position of the slot definition in the source code.
        position: TokenPosition,
    },

    /// An error indicating that multiple output slots were defined in a single
    /// widget definition.
    #[error("A widget cannot have multiple output slots defined: {widget} at {position}")]
    MultipleWidgetOutputsDefined {
        /// The name of the widget with multiple outputs.
        widget: String,

        /// The position of the widget definition in the source code.
        position: TokenPosition,
    },

    /// An error indicating that a layout has no output slot.
    #[error("Layout has no output slot: {widget} at {position}")]
    LayoutHasNoOutput {
        /// The name of the widget whose layout has no output.
        widget: String,

        /// The position of the widget definition in the source code.
        position: TokenPosition,
    },
}
