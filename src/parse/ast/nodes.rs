//! Nodes defined within the AST.

use bevy::color::Color;
use bevy::platform::collections::HashMap;

/// Represents a module node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ModuleNode<'a> {
    /// The list of import nodes in the module.
    pub imports: Vec<ImportNode<'a>>,

    /// The list of global variable assignments in the module.
    pub variables: Vec<VarAssignNode<'a>>,

    /// The list of layout nodes in the module.
    pub layouts: Vec<LayoutNode<'a>>,

    /// The list of style nodes in the module.
    pub styles: Vec<StyleNode<'a>>,

    /// The list of widget nodes in the module.
    pub widgets: Vec<WidgetNode<'a>>,
}

/// Represents an import node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ImportNode<'a> {
    /// The path of the imported module.
    pub path: &'a str,
}

/// Represents a variable in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Variable<'a> {
    /// The name of the variable.
    pub name: &'a str,
}

/// Represents a variable assignment node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VarAssignNode<'a> {
    /// The variable being assigned.
    pub variable: Variable<'a>,

    /// The type of the variable.
    pub var_type: VariableType,

    /// The expression assigned to the variable.
    pub expr: ExprNode<'a>,
}

/// Represents the type of a variable in the AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum VariableType {
    /// A standard variable.
    Variable,

    /// A property variable.
    Property,

    /// A constant variable.
    Constant,
}

/// Represents a constant node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ConstantNode<'a> {
    /// A string constant.
    String(&'a str),

    /// A boolean constant.
    Boolean(bool),

    /// A numeric constant.
    Number(f64),

    /// A color constant.
    Color(Color),

    /// A pixel constant.
    Pixels(f64),

    /// A percent constant.
    Percent(f64),

    /// A list constant.
    List(Vec<ExprNode<'a>>),

    /// A dictionary constant.
    Dict(Vec<(PropertyName<'a>, ExprNode<'a>)>),
}

/// Represents an expression node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ExprNode<'a> {
    /// An if-else expression.
    IfElse {
        /// The condition expression.
        condition: Box<ExprNode<'a>>,

        /// The expression if the condition is true.
        then_expr: Box<ExprNode<'a>>,

        /// The expression if the condition is false.
        else_expr: Box<ExprNode<'a>>,
    },

    /// A constant expression.
    UnaryExpr {
        /// The unary operator.
        operator: UnaryOperator,

        /// The expression.
        expr: Box<ExprNode<'a>>,
    },

    /// A binary expression.
    BinaryExpr {
        /// The binary operator.
        operator: BinaryOperator,

        /// The left expression.
        left: Box<ExprNode<'a>>,

        /// The right expression.
        right: Box<ExprNode<'a>>,
    },

    /// A term.
    Term(TermNode<'a>),
}

/// Represents a unary operator in the AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum UnaryOperator {
    /// Logical NOT operator.
    Not,

    /// Unary plus operator.
    Plus,

    /// Unary minus operator.
    Minus,
}

/// Represents a binary operator in the AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BinaryOperator {
    /// Addition operator.
    Add,

    /// Subtraction operator.
    Subtract,

    /// Multiplication operator.
    Multiply,

    /// Division operator.
    Divide,

    /// Exponentiation operator.
    Exponent,

    /// Range operator.
    Range,

    /// Inclusive range operator.
    InclusiveRange,

    /// Accessor operator.
    LessThanOrEqual,

    /// Greater than or equal operator.
    GreaterThanOrEqual,

    /// Equality operator.
    Equal,

    /// Inequality operator.
    NotEqual,

    /// Less than operator.
    LessThan,

    /// Greater than operator.
    GreaterThan,

    /// Logical AND operator.
    And,

    /// Logical OR operator.
    Or,
}

/// Represents a term node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TermNode<'a> {
    /// A variable term.
    Variable(Variable<'a>),

    /// A constant term.
    Constant(ConstantNode<'a>),

    /// A grouped expression term.
    Group(Box<ExprNode<'a>>),

    /// A function call term.
    FunctionCall {
        /// The name of the function.
        name: &'a str,

        /// The arguments passed to the function.
        args: Vec<ExprNode<'a>>,
    },

    /// A property/list/dict access term.
    Access {
        /// The base term being accessed.
        base: Box<TermNode<'a>>,

        /// The accessor applied to the base term.
        accessors: Box<ExprNode<'a>>,
    },
}

/// Represents a property name in the AST.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PropertyName<'a> {
    /// The name of the property.
    pub name: &'a str,
}

/// Represents an output name in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OutputName<'a> {
    /// The name of the output.
    pub name: &'a str,
}

impl OutputName<'static> {
    /// The default output name.
    pub const DEFAULT: Self = OutputName { name: "default" };
}

/// Represents a widget name in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WidgetName<'a> {
    /// The name of the widget.
    pub name: &'a str,
}

/// Represents a layout node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LayoutNode<'a> {
    /// The widget name.
    pub widget: WidgetName<'a>,

    /// The list of widget modifiers.
    pub modifiers: Vec<WidgetModifier<'a>>,

    /// The list of classes.
    pub classes: Vec<ClassName<'a>>,

    /// The list of properties.
    pub properties: HashMap<PropertyName<'a>, ExprNode<'a>>,

    /// The list of child layout nodes.
    pub children: Vec<LayoutNode<'a>>,
}

/// Represents a layout node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WidgetLayoutNode<'a> {
    /// The widget name.
    pub widget: WidgetName<'a>,

    /// The list of widget modifiers.
    pub modifiers: Vec<WidgetModifier<'a>>,

    /// The list of classes.
    pub classes: Vec<ClassName<'a>>,

    /// The list of properties.
    pub properties: HashMap<PropertyName<'a>, ExprNode<'a>>,

    /// The list of child layout nodes.
    pub children: Vec<WidgetLayoutNode<'a>>,

    /// The optional output name.
    ///
    /// If the node does not produce an output, this will be `None`.
    pub output: Option<OutputName<'a>>,
}

/// Represents a widget modifier in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum WidgetModifier<'a> {
    /// An if-condition modifier.
    If(ExprNode<'a>),

    /// A for-loop modifier.
    In(OutputName<'a>),

    /// A for-loop modifier.
    For {
        /// The loop variable.
        variable: Variable<'a>,

        /// The iterable expression.
        iterable: ExprNode<'a>,
    },

    /// A map modifier.
    Map {
        /// The mapping expression.
        mapping: ExprNode<'a>,
    },
}

/// Represents a style body item in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StyleNode<'a> {
    /// The widget this style applies to.
    pub widget: StyleWidget<'a>,

    /// The list of style selectors.
    pub selectors: Vec<StyleSelectorNode<'a>>,

    /// The optional in-modifier.
    pub in_modifier: OutputName<'a>,

    /// The body of the style.
    pub body: StyleBodyNode<'a>,
}

/// Represents a style body item in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StyleBodyNode<'a> {
    /// The property name.
    pub properties: HashMap<PropertyName<'a>, ExprNode<'a>>,

    /// The list of nested styles.
    pub nested_styles: Vec<StyleNode<'a>>,
}

/// Represents a style widget in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StyleWidget<'a> {
    /// A specific widget.
    Specific(WidgetName<'a>),

    /// A wildcard widget.
    Any,
}

/// Represents a style body item in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ClassName<'a> {
    /// The name of the class.
    pub name: &'a str,
}

/// Represents a style selector in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StyleSelectorNode<'a> {
    /// A whitelist class selector.
    WithClass(ClassName<'a>),

    /// A blacklist class selector.
    WithoutClass(ClassName<'a>),
}

/// Represents a widget node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WidgetNode<'a> {
    /// The name of the widget.
    pub name: WidgetName<'a>,

    /// The list of variable assignments.
    pub variables: Vec<VarAssignNode<'a>>,

    /// The layout of the widget.
    pub layout: WidgetLayoutNode<'a>,
}
