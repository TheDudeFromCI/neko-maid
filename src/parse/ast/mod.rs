#![allow(dead_code)]

//! This module implements a basic Abstract Syntax Tree (AST) structure for Neko
//! Maid UI files.

use bevy::color::Srgba;
use bevy::platform::collections::HashMap;
use pest::Parser;
use pest::iterators::Pair;
use pest::pratt_parser::{Assoc, Op, PrattParser};

pub mod nodes;

#[cfg(test)]
mod tests;

use lazy_static::lazy_static;
use nodes::*;
use parser::{NekoMaidParser, Rule};

lazy_static! {
    /// The Pratt parser for handling operator precedence and associativity
    /// in expressions.
    static ref PRATT_PARSER: PrattParser<Rule> = {
        PrattParser::new()
            .op(Op::infix(Rule::BooleanOp, Assoc::Left))
            .op(Op::infix(Rule::CompareOp, Assoc::Left))
            .op(Op::infix(Rule::RangeOp, Assoc::Left))
            .op(Op::infix(Rule::AddSub, Assoc::Left))
            .op(Op::infix(Rule::MulDiv, Assoc::Left))
            .op(Op::infix(Rule::Pow, Assoc::Right))
            .op(Op::prefix(Rule::Invert))
            .op(Op::prefix(Rule::NumberSign))
    };
}

/// The parser module contains the Pest parser definition for NekoMaid UI files.
///
/// The parser is generated from the grammar defined in `grammar.pest`, so it
/// needs to be in this module to suppress documentation warnings.
mod parser {
    #![allow(missing_docs)]

    use pest_derive::Parser;

    /// The AST represents the hierarchical structure of the nodes used in
    /// NekoMaid UI files.
    #[derive(Debug, Parser)]
    #[grammar = "parse/ast/grammar.pest"]
    pub struct NekoMaidParser;
}

/// Parses the given NekoMaid UI code into an AST ModuleNode.
pub(crate) fn parse(code: &str) -> Result<ModuleNode<'_>, String> {
    let module_pair = match NekoMaidParser::parse(Rule::Module, code) {
        Ok(mut pairs) => pairs.next().expect("Expected a module"),
        Err(e) => return Err(format!("NekoMaid Error: {}", e)),
    };

    let mut module = ModuleNode {
        imports: vec![],
        variables: vec![],
        layouts: vec![],
        styles: vec![],
        widgets: vec![],
    };

    for pair in module_pair.into_inner() {
        match pair.as_rule() {
            Rule::Import => {
                let import_node = parse_import(pair);
                module.imports.push(import_node);
            }
            Rule::VarAssign | Rule::ConstAssign => {
                let var_assign_node = parse_var_assign(pair);
                module.variables.push(var_assign_node);
            }
            Rule::Layout => {
                let layout_node = parse_layout(pair);
                module.layouts.push(layout_node);
            }
            Rule::Style => {
                let style_node = parse_style(pair);
                module.styles.push(style_node);
            }
            Rule::Widget => {
                let widget_node = parse_widget(pair);
                module.widgets.push(widget_node);
            }
            Rule::EOI => {}
            r => panic!("Unexpected rule in module: {:?}", r),
        }
    }

    Ok(module)
}

/// Parses an import pair into an ImportNode.
fn parse_import(pair: Pair<'_, Rule>) -> ImportNode<'_> {
    match pair.as_rule() {
        Rule::Import => {
            let pair = pair.into_inner().next().unwrap();
            match pair.as_rule() {
                Rule::String => {
                    let path = parse_string(pair);
                    ImportNode { path }
                }
                r => panic!("Unexpected rule in import: {:?}", r),
            }
        }
        r => panic!("Unexpected rule in import: {:?}", r),
    }
}

/// Parses a variable assignment pair into a VarAssignNode.
fn parse_var_assign(pair: Pair<'_, Rule>) -> VarAssignNode<'_> {
    match pair.as_rule() {
        Rule::VarAssign => {
            let mut pair = pair.into_inner();
            let variable = parse_variable_name(pair.next().unwrap());
            let expr = parse_expr(pair.next().unwrap());
            VarAssignNode {
                variable,
                expr,
                var_type: VariableType::Variable,
            }
        }
        Rule::ConstAssign => {
            let mut pair = pair.into_inner();
            let variable = parse_variable_name(pair.next().unwrap());
            let expr = parse_expr(pair.next().unwrap());
            VarAssignNode {
                variable,
                expr,
                var_type: VariableType::Constant,
            }
        }
        Rule::PropertyVarAssign => {
            let mut pair = pair.into_inner();
            let variable = parse_variable_name(pair.next().unwrap());
            let expr = parse_expr(pair.next().unwrap());
            VarAssignNode {
                variable,
                expr,
                var_type: VariableType::Property,
            }
        }
        r => panic!("Unexpected rule in var assign: {:?}", r),
    }
}

/// Parses a variable assignment pair into a Variable.
fn parse_variable_name(pair: Pair<'_, Rule>) -> Variable<'_> {
    match pair.as_rule() {
        Rule::VariableName => Variable {
            name: pair.as_str(),
        },
        r => panic!("Unexpected rule in variable name: {:?}", r),
    }
}

/// Parses a variable reference pair into a Variable.
fn parse_variable_ref(pair: Pair<'_, Rule>) -> Variable<'_> {
    match pair.as_rule() {
        Rule::VariableRef => {
            let pair = pair.into_inner().next().unwrap();
            match pair.as_rule() {
                Rule::VariableName => parse_variable_name(pair),
                r => panic!("Unexpected rule in variable ref: {:?}", r),
            }
        }
        r => panic!("Unexpected rule in variable ref: {:?}", r),
    }
}

/// Parses a group pair into an ExprNode.
fn parse_group(pair: Pair<'_, Rule>) -> ExprNode<'_> {
    match pair.as_rule() {
        Rule::Group => {
            let expr_pair = pair.into_inner().next().unwrap();
            parse_expr(expr_pair)
        }
        r => panic!("Unexpected rule in group: {:?}", r),
    }
}

/// Parses an expression pair into an ExprNode.
fn parse_expr(pair: Pair<'_, Rule>) -> ExprNode<'_> {
    match pair.as_rule() {
        Rule::Expr => PRATT_PARSER
            .map_primary(|primary| match primary.as_rule() {
                Rule::Term => ExprNode::Term(parse_term(primary)),
                Rule::IfElseExpr => parse_if_else_expr(primary),
                r => panic!("Unexpected rule in expr primary: {:?}", r),
            })
            .map_prefix(|op, expr| {
                let operator = match op.as_rule() {
                    Rule::NumberSign => match op.as_str() {
                        "+" => UnaryOperator::Plus,
                        "-" => UnaryOperator::Minus,
                        r => panic!("Unexpected unary operator: {:?}", r),
                    },
                    Rule::Invert => match op.as_str() {
                        "not" | "!" => UnaryOperator::Not,
                        r => panic!("Unexpected unary operator: {:?}", r),
                    },
                    r => panic!("Unexpected rule in unary expr operator: {:?}", r),
                };

                ExprNode::UnaryExpr {
                    operator,
                    expr: Box::new(expr),
                }
            })
            .map_infix(|left, op, right| {
                let operator = match op.as_rule() {
                    Rule::AddSub => match op.as_str() {
                        "+" => BinaryOperator::Add,
                        "-" => BinaryOperator::Subtract,
                        r => panic!("Unexpected AddSub operator: {:?}", r),
                    },
                    Rule::MulDiv => match op.as_str() {
                        "*" => BinaryOperator::Multiply,
                        "/" => BinaryOperator::Divide,
                        r => panic!("Unexpected MulDiv operator: {:?}", r),
                    },
                    Rule::Pow => match op.as_str() {
                        "^" | "**" => BinaryOperator::Exponent,
                        r => panic!("Unexpected Pow operator: {:?}", r),
                    },
                    Rule::RangeOp => match op.as_str() {
                        ".." => BinaryOperator::Range,
                        "..=" => BinaryOperator::InclusiveRange,
                        r => panic!("Unexpected RangeOp operator: {:?}", r),
                    },
                    Rule::CompareOp => match op.as_str() {
                        "<=" => BinaryOperator::LessThanOrEqual,
                        ">=" => BinaryOperator::GreaterThanOrEqual,
                        "==" | "is" => BinaryOperator::Equal,
                        "!=" => BinaryOperator::NotEqual,
                        "<" => BinaryOperator::LessThan,
                        ">" => BinaryOperator::GreaterThan,
                        r => panic!("Unexpected CompareOp operator: {:?}", r),
                    },
                    Rule::BooleanOp => match op.as_str() {
                        "and" | "&&" => BinaryOperator::And,
                        "or" | "||" => BinaryOperator::Or,
                        r => panic!("Unexpected BooleanOp operator: {:?}", r),
                    },
                    r => panic!("Unexpected rule in binary expr operator: {:?}", r),
                };

                ExprNode::BinaryExpr {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            })
            .parse(pair.into_inner()),
        r => panic!("Unexpected rule in expr: {:?}", r),
    }
}

/// Parses a function call pair into an TermNode.
fn parse_function_call(pair: Pair<'_, Rule>) -> TermNode<'_> {
    match pair.as_rule() {
        Rule::FunctionCall => {
            let mut inner_pairs = pair.into_inner();
            let func_name_pair = inner_pairs.next().unwrap();
            let args_pair = inner_pairs.next().unwrap();

            let func_name = func_name_pair.as_str();
            let mut args = vec![];

            for arg_pair in args_pair.into_inner() {
                match arg_pair.as_rule() {
                    Rule::Expr => {
                        let arg_expr = parse_expr(arg_pair);
                        args.push(arg_expr);
                    }
                    r => panic!("Unexpected rule in function call args: {:?}", r),
                }
            }

            TermNode::FunctionCall {
                name: func_name,
                args,
            }
        }
        r => panic!("Unexpected rule in function call: {:?}", r),
    }
}

/// Parses a term pair into a TermNode.
fn parse_term(pair: Pair<'_, Rule>) -> TermNode<'_> {
    match pair.as_rule() {
        Rule::Term => {
            let mut pair = pair.into_inner();

            let term_pair = pair.next().unwrap();
            let mut term = match term_pair.as_rule() {
                Rule::VariableRef => {
                    let variable = parse_variable_ref(term_pair);
                    TermNode::Variable(variable)
                }
                Rule::Constant => {
                    let constant_node = parse_constant(term_pair);
                    TermNode::Constant(constant_node)
                }
                Rule::Group => {
                    let expr_node = parse_group(term_pair);
                    TermNode::Group(Box::new(expr_node))
                }
                Rule::FunctionCall => parse_function_call(term_pair),
                r => panic!("Unexpected rule in term: {:?}", r),
            };

            for accessor_pair in pair {
                match accessor_pair.as_rule() {
                    Rule::PropertyAccess => {
                        let name_pair = accessor_pair.into_inner().next().unwrap();
                        let expr = ExprNode::Term(TermNode::Constant(ConstantNode::String(
                            name_pair.as_str(),
                        )));
                        term = TermNode::Access {
                            base: Box::new(term),
                            accessors: Box::new(expr),
                        };
                    }
                    Rule::ListAccess => {
                        let expr_pair = accessor_pair.into_inner().next().unwrap();
                        let expr = parse_expr(expr_pair);
                        term = TermNode::Access {
                            base: Box::new(term),
                            accessors: Box::new(expr),
                        };
                    }
                    r => panic!("Unexpected rule in term accessors: {:?}", r),
                }
            }

            term
        }
        r => panic!("Unexpected rule in term: {:?}", r),
    }
}

/// Parses an if-else expression pair into an ExprNode.
fn parse_if_else_expr(pair: Pair<'_, Rule>) -> ExprNode<'_> {
    match pair.as_rule() {
        Rule::IfElseExpr => {
            let mut inner_pairs = pair.into_inner();
            let condition_pair = inner_pairs.next().unwrap();
            let then_pair = inner_pairs.next().unwrap();
            let else_pair = inner_pairs.next().unwrap();

            let condition = parse_expr(condition_pair);
            let then_expr = parse_expr(then_pair);
            let else_expr = parse_expr(else_pair);

            ExprNode::IfElse {
                condition: Box::new(condition),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            }
        }
        r => panic!("Unexpected rule in if-else expr: {:?}", r),
    }
}

/// Parses a constant pair into a ConstantNode.
fn parse_constant(pair: Pair<'_, Rule>) -> ConstantNode<'_> {
    match pair.as_rule() {
        Rule::Constant => {
            let pair = pair.into_inner().next().unwrap();
            match pair.as_rule() {
                Rule::String => ConstantNode::String(parse_string(pair)),
                Rule::Boolean => {
                    let value = match pair.as_str() {
                        "true" => true,
                        "false" => false,
                        r => panic!("Unexpected rule in bool: {:?}", r),
                    };
                    ConstantNode::Boolean(value)
                }
                Rule::Number => {
                    let value: f64 = pair.as_str().parse().unwrap();
                    ConstantNode::Number(value)
                }
                Rule::Color => {
                    let value = Srgba::hex(pair.as_str()).unwrap();
                    ConstantNode::Color(value.into())
                }
                Rule::Pixels => {
                    let num_pair = pair.into_inner().next().unwrap();
                    let value: f64 = num_pair.as_str().parse().unwrap();
                    ConstantNode::Pixels(value)
                }
                Rule::Percent => {
                    let num_pair = pair.into_inner().next().unwrap();
                    let value: f64 = num_pair.as_str().parse().unwrap();
                    ConstantNode::Percent(value)
                }
                Rule::List => {
                    let mut elements = vec![];
                    for list_pairs in pair.into_inner() {
                        match list_pairs.as_rule() {
                            Rule::Expr => {
                                let expr = parse_expr(list_pairs);
                                elements.push(expr);
                            }
                            r => panic!("Unexpected rule in list: {:?}", r),
                        }
                    }
                    ConstantNode::List(elements)
                }
                Rule::Dict => {
                    let mut entries = vec![];
                    for kv_pair in pair.into_inner() {
                        let mut kv_inner = kv_pair.into_inner();
                        let key_pair = kv_inner.next().unwrap();
                        let value_pair = kv_inner.next().unwrap();

                        let key = PropertyName {
                            name: key_pair.as_str(),
                        };
                        let value = parse_expr(value_pair);

                        entries.push((key, value));
                    }
                    ConstantNode::Dict(entries)
                }
                r => panic!("Unexpected rule in constant: {:?}", r),
            }
        }
        r => panic!("Unexpected rule in constant: {:?}", r),
    }
}

/// Parses a string pair into a &str.
fn parse_string(pair: Pair<'_, Rule>) -> &'_ str {
    match pair.as_rule() {
        Rule::String => {
            let pair = pair.into_inner().next().unwrap();
            match pair.as_rule() {
                Rule::DoubleQuoteStr | Rule::SingleQuoteStr | Rule::TickQuoteStr => {
                    let inner_str = pair.into_inner().next().unwrap();
                    inner_str.as_str()
                }
                Rule::SimpleString => pair.as_str(),
                r => panic!("Unexpected rule in string: {:?}", r),
            }
        }
        Rule::DoubleQuoteStr | Rule::SingleQuoteStr | Rule::TickQuoteStr => {
            let inner_str = pair.into_inner().next().unwrap();
            inner_str.as_str()
        }
        Rule::SimpleString => pair.as_str(),
        r => panic!("Unexpected rule in string: {:?}", r),
    }
}

/// Parses a layout pair into a LayoutNode.
fn parse_layout(pair: Pair<'_, Rule>) -> LayoutNode<'_> {
    match pair.as_rule() {
        Rule::Layout => {
            let mut inner_pairs = pair.into_inner();
            let widget_name_pair = inner_pairs.next().unwrap();
            let widget_name = WidgetName {
                name: widget_name_pair.as_str(),
            };

            let modifiers_pair = inner_pairs.next().unwrap();
            let modifiers = parse_layout_modifiers(modifiers_pair);

            let mut classes = vec![];
            let mut properties = HashMap::new();
            let mut children = vec![];

            let Some(body_pair) = inner_pairs.next() else {
                return LayoutNode {
                    widget: widget_name,
                    modifiers,
                    classes,
                    properties,
                    children,
                };
            };

            match body_pair.as_rule() {
                Rule::LayoutBody => {
                    for body_item_pair in body_pair.into_inner() {
                        match body_item_pair.as_rule() {
                            Rule::ClassAssign => {
                                let class_name_pair = body_item_pair.into_inner().next().unwrap();
                                let class_name = ClassName {
                                    name: class_name_pair.as_str(),
                                };
                                classes.push(class_name);
                            }
                            Rule::Layout => {
                                let child_layout = parse_layout(body_item_pair);
                                children.push(child_layout);
                            }
                            Rule::PropertyAssign => {
                                let mut prop_inner = body_item_pair.into_inner();
                                let name_pair = prop_inner.next().unwrap();
                                let expr_pair = prop_inner.next().unwrap();

                                let name = PropertyName {
                                    name: name_pair.as_str(),
                                };
                                let expr = parse_expr(expr_pair);

                                properties.insert(name, expr);
                            }
                            r => panic!("Unexpected rule in layout body: {:?}", r),
                        }
                    }
                }
                r => panic!("Unexpected rule in layout body: {:?}", r),
            }

            LayoutNode {
                widget: widget_name,
                modifiers,
                classes,
                properties,
                children,
            }
        }
        r => panic!("Unexpected rule in layout: {:?}", r),
    }
}

/// Parses layout modifiers into a vector of WidgetModifier.
fn parse_layout_modifiers(pair: Pair<'_, Rule>) -> Vec<WidgetModifier<'_>> {
    let mut modifiers = vec![];

    match pair.as_rule() {
        Rule::LayoutModifiers => {
            for modifier_pair in pair.into_inner() {
                match modifier_pair.as_rule() {
                    Rule::InModifier => {
                        let output_name_pair = modifier_pair.into_inner().next().unwrap();
                        let name = OutputName {
                            name: output_name_pair.as_str(),
                        };
                        modifiers.push(WidgetModifier::In(name));
                    }
                    Rule::IfModifier => {
                        let expr_pair = modifier_pair.into_inner().next().unwrap();
                        let expr = parse_expr(expr_pair);
                        modifiers.push(WidgetModifier::If(expr));
                    }
                    Rule::ForModifier => {
                        let mut for_inner = modifier_pair.into_inner();
                        let var_ref_pair = for_inner.next().unwrap();
                        let expr_pair = for_inner.next().unwrap();

                        let variable = parse_variable_ref(var_ref_pair);
                        let iterable = parse_expr(expr_pair);

                        modifiers.push(WidgetModifier::For { variable, iterable });
                    }
                    Rule::MapModifier => {
                        let expr_pair = modifier_pair.into_inner().next().unwrap();
                        let mapping = parse_expr(expr_pair);
                        modifiers.push(WidgetModifier::Map { mapping });
                    }
                    r => panic!("Unexpected rule in widget modifiers: {:?}", r),
                }
            }
        }
        r => panic!("Unexpected rule in widget modifiers: {:?}", r),
    }

    modifiers
}

/// Parses a style pair into a StyleNode.
fn parse_style(pair: Pair<'_, Rule>) -> StyleNode<'_> {
    match pair.as_rule() {
        Rule::Style => {
            let mut inner_pairs = pair.into_inner();

            let widget_pair = inner_pairs.next().unwrap();
            let widget = match widget_pair.as_str() {
                "*" => StyleWidget::Any,
                r => StyleWidget::Specific(WidgetName { name: r }),
            };

            let selector_pair = inner_pairs.next().unwrap();
            let selectors = parse_selectors(selector_pair);

            let mut in_modifier = OutputName::DEFAULT;
            if let Some(Rule::InModifier) = inner_pairs.peek().map(|p| p.as_rule()) {
                let in_modifier_pair = inner_pairs.next().unwrap();
                let output_name_pair = in_modifier_pair.into_inner().next().unwrap();
                in_modifier = OutputName {
                    name: output_name_pair.as_str(),
                };
            };

            let body_pair = inner_pairs.next().unwrap();
            let body = parse_style_body(body_pair);

            StyleNode {
                widget,
                selectors,
                in_modifier,
                body,
            }
        }
        r => panic!("Unexpected rule in style: {:?}", r),
    }
}

/// Parses a style body pair into a StyleBodyNode.
fn parse_style_body(pair: Pair<'_, Rule>) -> StyleBodyNode<'_> {
    let mut body = StyleBodyNode {
        properties: HashMap::new(),
        nested_styles: vec![],
    };

    match pair.as_rule() {
        Rule::StyleBody => {
            for body_item_pair in pair.into_inner() {
                match body_item_pair.as_rule() {
                    Rule::PropertyAssign => {
                        let mut prop_inner = body_item_pair.into_inner();

                        let name_pair = prop_inner.next().unwrap();
                        let name = PropertyName {
                            name: name_pair.as_str(),
                        };

                        let expr_pair = prop_inner.next().unwrap();
                        let expr = parse_expr(expr_pair);

                        body.properties.insert(name, expr);
                    }
                    Rule::Style => {
                        let child_style = parse_style(body_item_pair);
                        body.nested_styles.push(child_style);
                    }
                    r => panic!("Unexpected rule in style body: {:?}", r),
                }
            }
        }
        r => panic!("Unexpected rule in style body: {:?}", r),
    }

    body
}

/// Parses style selectors into a vector of StyleSelectorNode.
fn parse_selectors(pair: Pair<'_, Rule>) -> Vec<StyleSelectorNode<'_>> {
    let mut selectors = vec![];

    match pair.as_rule() {
        Rule::StyleSelector => {
            let pairs = pair.into_inner();
            for selector_pair in pairs {
                match selector_pair.as_rule() {
                    Rule::WithClass => {
                        let class_name_pair = selector_pair.into_inner().next().unwrap();
                        let class_name = ClassName {
                            name: class_name_pair.as_str(),
                        };
                        selectors.push(StyleSelectorNode::WithClass(class_name));
                    }
                    Rule::WithoutClass => {
                        let class_name_pair = selector_pair.into_inner().next().unwrap();
                        let class_name = ClassName {
                            name: class_name_pair.as_str(),
                        };
                        selectors.push(StyleSelectorNode::WithoutClass(class_name));
                    }
                    r => panic!("Unexpected rule in style selector: {:?}", r),
                }
            }
        }
        r => panic!("Unexpected rule in style selectors: {:?}", r),
    }

    selectors
}

/// Parses a widget pair into a WidgetNode.
fn parse_widget(pair: Pair<'_, Rule>) -> WidgetNode<'_> {
    match pair.as_rule() {
        Rule::Widget => {
            let mut inner_pairs = pair.into_inner();
            let widget_name_pair = inner_pairs.next().unwrap();
            let widget_name = WidgetName {
                name: widget_name_pair.as_str(),
            };

            let widget_header_pair = inner_pairs.next().unwrap();
            let variables = parse_widget_header(widget_header_pair);

            let widget_layout_pair = inner_pairs.next().unwrap();
            let layout = parse_widget_layout(widget_layout_pair);

            WidgetNode {
                name: widget_name,
                variables,
                layout,
            }
        }
        r => panic!("Unexpected rule in widget: {:?}", r),
    }
}

/// Parses a widget header pair into a vector of VarAssignNode.
fn parse_widget_header(pair: Pair<'_, Rule>) -> Vec<VarAssignNode<'_>> {
    let mut variables = vec![];

    match pair.as_rule() {
        Rule::WidgetHeader => {
            for body_item_pair in pair.into_inner() {
                let var_assign_node = parse_var_assign(body_item_pair);
                variables.push(var_assign_node);
            }
        }
        r => panic!("Unexpected rule in widget header: {:?}", r),
    }

    variables
}

/// Parses a widget layout pair into a WidgetLayoutNode.
fn parse_widget_layout(pair: Pair<'_, Rule>) -> WidgetLayoutNode<'_> {
    match pair.as_rule() {
        Rule::WidgetLayout => {
            let mut inner_pairs = pair.into_inner();
            let widget_name_pair = inner_pairs.next().unwrap();
            let widget_name = WidgetName {
                name: widget_name_pair.as_str(),
            };

            let modifiers_pair = inner_pairs.next().unwrap();
            let modifiers = parse_layout_modifiers(modifiers_pair);

            let mut classes = vec![];
            let mut properties = HashMap::new();
            let mut children = vec![];
            let mut output = None;

            let Some(body_pair) = inner_pairs.next() else {
                return WidgetLayoutNode {
                    widget: widget_name,
                    modifiers,
                    classes,
                    properties,
                    children,
                    output,
                };
            };

            match body_pair.as_rule() {
                Rule::WidgetLayoutBody => {
                    for body_item_pair in body_pair.into_inner() {
                        match body_item_pair.as_rule() {
                            Rule::ClassAssign => {
                                let class_name_pair = body_item_pair.into_inner().next().unwrap();
                                let class_name = ClassName {
                                    name: class_name_pair.as_str(),
                                };
                                classes.push(class_name);
                            }
                            Rule::WidgetLayout => {
                                let child_layout = parse_widget_layout(body_item_pair);
                                children.push(child_layout);
                            }
                            Rule::PropertyAssign => {
                                let mut prop_inner = body_item_pair.into_inner();
                                let name_pair = prop_inner.next().unwrap();
                                let expr_pair = prop_inner.next().unwrap();

                                let name = PropertyName {
                                    name: name_pair.as_str(),
                                };
                                let expr = parse_expr(expr_pair);

                                properties.insert(name, expr);
                            }
                            Rule::OutputAssign => {
                                output = Some(parse_output(body_item_pair));
                            }
                            r => panic!("Unexpected rule in widget layout body: {:?}", r),
                        }
                    }
                }
                r => panic!("Unexpected rule in widget layout body: {:?}", r),
            }

            WidgetLayoutNode {
                widget: widget_name,
                modifiers,
                classes,
                properties,
                children,
                output,
            }
        }
        r => panic!("Unexpected rule in widget layout: {:?}", r),
    }
}

/// Parses an output name pair into an OutputName.
fn parse_output(pair: Pair<'_, Rule>) -> OutputName<'_> {
    match pair.as_rule() {
        Rule::OutputAssign => {
            let mut name = OutputName::DEFAULT;

            if let Some(output_name_pair) = pair.into_inner().next() {
                name = OutputName {
                    name: output_name_pair.as_str(),
                };
            }

            name
        }
        r => panic!("Unexpected rule in output name: {:?}", r),
    }
}
