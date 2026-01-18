//! Tests

use bevy::color::Color;
use bevy::platform::collections::HashMap;
use pretty_assertions::assert_eq;

use crate::parse::ast;
use crate::parse::ast::nodes::*;

fn parse(code: &str) -> ModuleNode<'_> {
    match ast::parse(code) {
        Ok(module) => module,
        Err(e) => panic!("Failed to parse code: {}", e),
    }
}

#[test]
fn test_parse_import() {
    let code = r#"
            import "ui/common.neko";
        "#;

    let module = parse(code);
    assert_eq!(
        module.imports,
        vec![ImportNode {
            path: "ui/common.neko"
        }]
    );
}

#[test]
fn test_assign_constants() {
    let code = r#"
            var title = "Hello, NekoMaid!";
            var is_visible = true;
            var margin = auto;
            var width = 200px;
            var color = #FF0000;
            var percentage = 75%;
            var items = ["Item 1", "Item 2", "Item 3"];
            var settings = { theme: "dark", notifications: false };

            var nested_list = [1, [2, 3], { key: "value" }];
            var nested_dict = { outer_key: { inner_key: 42 } };
        "#;

    let module = parse(code);
    assert_eq!(
        module.variables,
        vec![
            VarAssignNode {
                variable: Variable { name: "title" },
                expr: ExprNode::Term(TermNode::Constant(ConstantNode::String("Hello, NekoMaid!"))),
                var_type: VariableType::Variable,
            },
            VarAssignNode {
                variable: Variable { name: "is_visible" },
                expr: ExprNode::Term(TermNode::Constant(ConstantNode::Boolean(true))),
                var_type: VariableType::Variable,
            },
            VarAssignNode {
                variable: Variable { name: "margin" },
                expr: ExprNode::Term(TermNode::Constant(ConstantNode::String("auto"))),
                var_type: VariableType::Variable,
            },
            VarAssignNode {
                variable: Variable { name: "width" },
                expr: ExprNode::Term(TermNode::Constant(ConstantNode::Pixels(200.0))),
                var_type: VariableType::Variable,
            },
            VarAssignNode {
                variable: Variable { name: "color" },
                expr: ExprNode::Term(TermNode::Constant(ConstantNode::Color(Color::srgb(
                    1.0, 0.0, 0.0
                )))),
                var_type: VariableType::Variable,
            },
            VarAssignNode {
                variable: Variable { name: "percentage" },
                expr: ExprNode::Term(TermNode::Constant(ConstantNode::Percent(75.0))),
                var_type: VariableType::Variable,
            },
            VarAssignNode {
                variable: Variable { name: "items" },
                expr: ExprNode::Term(TermNode::Constant(ConstantNode::List(vec![
                    ExprNode::Term(TermNode::Constant(ConstantNode::String("Item 1"))),
                    ExprNode::Term(TermNode::Constant(ConstantNode::String("Item 2"))),
                    ExprNode::Term(TermNode::Constant(ConstantNode::String("Item 3"))),
                ]))),
                var_type: VariableType::Variable,
            },
            VarAssignNode {
                variable: Variable { name: "settings" },
                expr: ExprNode::Term(TermNode::Constant(ConstantNode::Dict(vec![
                    (
                        PropertyName { name: "theme" },
                        ExprNode::Term(TermNode::Constant(ConstantNode::String("dark"))),
                    ),
                    (
                        PropertyName {
                            name: "notifications"
                        },
                        ExprNode::Term(TermNode::Constant(ConstantNode::Boolean(false))),
                    ),
                ]))),
                var_type: VariableType::Variable,
            },
            VarAssignNode {
                variable: Variable {
                    name: "nested_list"
                },
                expr: ExprNode::Term(TermNode::Constant(ConstantNode::List(vec![
                    ExprNode::Term(TermNode::Constant(ConstantNode::Number(1.0))),
                    ExprNode::Term(TermNode::Constant(ConstantNode::List(vec![
                        ExprNode::Term(TermNode::Constant(ConstantNode::Number(2.0))),
                        ExprNode::Term(TermNode::Constant(ConstantNode::Number(3.0))),
                    ]))),
                    ExprNode::Term(TermNode::Constant(ConstantNode::Dict(vec![(
                        PropertyName { name: "key" },
                        ExprNode::Term(TermNode::Constant(ConstantNode::String("value"))),
                    )]))),
                ]))),
                var_type: VariableType::Variable,
            },
            VarAssignNode {
                variable: Variable {
                    name: "nested_dict"
                },
                expr: ExprNode::Term(TermNode::Constant(ConstantNode::Dict(vec![(
                    PropertyName { name: "outer_key" },
                    ExprNode::Term(TermNode::Constant(ConstantNode::Dict(vec![(
                        PropertyName { name: "inner_key" },
                        ExprNode::Term(TermNode::Constant(ConstantNode::Number(42.0))),
                    ),]))),
                )]))),
                var_type: VariableType::Variable,
            },
        ]
    );
}

#[test]
fn test_simple_layout() {
    let code = r#"
            layout div {
                class main-container;

                width: 100%;
                height: 200px;
                background-color: #00FF00;

                with p {
                    text: "My favorite color is ";

                    with span {
                        text: "blue";
                        color: #0000FF;
                    }

                    with span {
                        text: ".";
                    }
                }
            }
        "#;

    let module = parse(code);
    assert_eq!(
        module,
        ModuleNode {
            imports: vec![],
            variables: vec![],
            layouts: vec![LayoutNode {
                widget: WidgetName { name: "div" },
                modifiers: vec![],
                classes: vec![ClassName {
                    name: "main-container"
                }],
                properties: {
                    let mut props = HashMap::new();
                    props.insert(
                        PropertyName { name: "width" },
                        ExprNode::Term(TermNode::Constant(ConstantNode::Percent(100.0))),
                    );
                    props.insert(
                        PropertyName { name: "height" },
                        ExprNode::Term(TermNode::Constant(ConstantNode::Pixels(200.0))),
                    );
                    props.insert(
                        PropertyName {
                            name: "background-color",
                        },
                        ExprNode::Term(TermNode::Constant(ConstantNode::Color(Color::srgb(
                            0.0, 1.0, 0.0,
                        )))),
                    );
                    props
                },
                children: vec![LayoutNode {
                    widget: WidgetName { name: "p" },
                    modifiers: vec![],
                    classes: vec![],
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            PropertyName { name: "text" },
                            ExprNode::Term(TermNode::Constant(ConstantNode::String(
                                "My favorite color is ",
                            ))),
                        );
                        props
                    },
                    children: vec![
                        LayoutNode {
                            widget: WidgetName { name: "span" },
                            modifiers: vec![],
                            classes: vec![],
                            properties: {
                                let mut props = HashMap::new();
                                props.insert(
                                    PropertyName { name: "text" },
                                    ExprNode::Term(TermNode::Constant(ConstantNode::String(
                                        "blue",
                                    ))),
                                );
                                props.insert(
                                    PropertyName { name: "color" },
                                    ExprNode::Term(TermNode::Constant(ConstantNode::Color(
                                        Color::srgb(0.0, 0.0, 1.0),
                                    ))),
                                );
                                props
                            },
                            children: vec![],
                        },
                        LayoutNode {
                            widget: WidgetName { name: "span" },
                            modifiers: vec![],
                            classes: vec![],
                            properties: {
                                let mut props = HashMap::new();
                                props.insert(
                                    PropertyName { name: "text" },
                                    ExprNode::Term(TermNode::Constant(ConstantNode::String("."))),
                                );
                                props
                            },
                            children: vec![],
                        },
                    ],
                }],
            }],
            styles: vec![],
            widgets: vec![],
        }
    );
}

#[test]
fn test_order_of_operations() {
    let code = r#"
            var nine = 3 + 48 * 2 / (1 - 5) ^ 2;
            var range = 1*5..=10;
            var access = $myDict.key1["prop" + 2][3] * 4;
            var invert = not (13 > 7 and false);
        "#;

    let module = parse(code);
    assert_eq!(
        module.variables,
        vec![
            VarAssignNode {
                variable: Variable { name: "nine" },
                expr: ExprNode::BinaryExpr {
                    operator: BinaryOperator::Add,
                    left: Box::new(ExprNode::Term(TermNode::Constant(ConstantNode::Number(
                        3.0
                    )))),
                    right: Box::new(ExprNode::BinaryExpr {
                        operator: BinaryOperator::Divide,
                        left: Box::new(ExprNode::BinaryExpr {
                            operator: BinaryOperator::Multiply,
                            left: Box::new(ExprNode::Term(TermNode::Constant(
                                ConstantNode::Number(48.0)
                            ))),
                            right: Box::new(ExprNode::Term(TermNode::Constant(
                                ConstantNode::Number(2.0)
                            ))),
                        }),
                        right: Box::new(ExprNode::BinaryExpr {
                            operator: BinaryOperator::Exponent,
                            left: Box::new(ExprNode::Term(TermNode::Group(Box::new(
                                ExprNode::BinaryExpr {
                                    operator: BinaryOperator::Subtract,
                                    left: Box::new(ExprNode::Term(TermNode::Constant(
                                        ConstantNode::Number(1.0)
                                    ))),
                                    right: Box::new(ExprNode::Term(TermNode::Constant(
                                        ConstantNode::Number(5.0)
                                    ))),
                                }
                            )))),
                            right: Box::new(ExprNode::Term(TermNode::Constant(
                                ConstantNode::Number(2.0)
                            ))),
                        }),
                    }),
                },
                var_type: VariableType::Variable,
            },
            VarAssignNode {
                variable: Variable { name: "range" },
                expr: ExprNode::BinaryExpr {
                    operator: BinaryOperator::InclusiveRange,
                    left: Box::new(ExprNode::BinaryExpr {
                        operator: BinaryOperator::Multiply,
                        left: Box::new(ExprNode::Term(TermNode::Constant(ConstantNode::Number(
                            1.0
                        )))),
                        right: Box::new(ExprNode::Term(TermNode::Constant(ConstantNode::Number(
                            5.0
                        )))),
                    }),
                    right: Box::new(ExprNode::Term(TermNode::Constant(ConstantNode::Number(
                        10.0
                    )))),
                },
                var_type: VariableType::Variable,
            },
            VarAssignNode {
                variable: Variable { name: "access" },
                expr: ExprNode::BinaryExpr {
                    operator: BinaryOperator::Multiply,
                    left: Box::new(ExprNode::Term(TermNode::Access {
                        base: Box::new(TermNode::Access {
                            base: Box::new(TermNode::Access {
                                base: Box::new(TermNode::Variable(Variable { name: "myDict" })),
                                accessors: Box::new(ExprNode::Term(TermNode::Constant(
                                    ConstantNode::String("key1")
                                ))),
                            }),
                            accessors: Box::new(ExprNode::BinaryExpr {
                                operator: BinaryOperator::Add,
                                left: Box::new(ExprNode::Term(TermNode::Constant(
                                    ConstantNode::String("prop")
                                ))),
                                right: Box::new(ExprNode::Term(TermNode::Constant(
                                    ConstantNode::Number(2.0)
                                ))),
                            }),
                        }),
                        accessors: Box::new(ExprNode::Term(TermNode::Constant(
                            ConstantNode::Number(3.0)
                        ))),
                    })),
                    right: Box::new(ExprNode::Term(TermNode::Constant(ConstantNode::Number(
                        4.0
                    )))),
                },
                var_type: VariableType::Variable,
            },
            VarAssignNode {
                variable: Variable { name: "invert" },
                expr: ExprNode::UnaryExpr {
                    operator: UnaryOperator::Not,
                    expr: Box::new(ExprNode::Term(TermNode::Group(Box::new(
                        ExprNode::BinaryExpr {
                            operator: BinaryOperator::And,
                            left: Box::new(ExprNode::BinaryExpr {
                                operator: BinaryOperator::GreaterThan,
                                left: Box::new(ExprNode::Term(TermNode::Constant(
                                    ConstantNode::Number(13.0)
                                ))),
                                right: Box::new(ExprNode::Term(TermNode::Constant(
                                    ConstantNode::Number(7.0)
                                ))),
                            }),
                            right: Box::new(ExprNode::Term(TermNode::Constant(
                                ConstantNode::Boolean(false)
                            ))),
                        }
                    )))),
                },
                var_type: VariableType::Variable,
            }
        ]
    );
}

#[test]
fn test_simple_widget() {
    let code = r#"
            def h1 {
                property text = "";
                property exclaim = true;

                layout p {
                    class h1;

                    text: $text;
                    font-size: 24px;

                    with span if $exclaim {
                        color: #0f0;
                        text: "!";
                    }

                    output;
                }
            }
        "#;

    let module = parse(code);
    assert_eq!(
        module,
        ModuleNode {
            imports: vec![],
            variables: vec![],
            layouts: vec![],
            styles: vec![],
            widgets: vec![WidgetNode {
                name: WidgetName { name: "h1" },
                variables: vec![
                    VarAssignNode {
                        variable: Variable { name: "text" },
                        expr: ExprNode::Term(TermNode::Constant(ConstantNode::String(""))),
                        var_type: VariableType::Property,
                    },
                    VarAssignNode {
                        variable: Variable { name: "exclaim" },
                        expr: ExprNode::Term(TermNode::Constant(ConstantNode::Boolean(true))),
                        var_type: VariableType::Property,
                    }
                ],
                layout: WidgetLayoutNode {
                    widget: WidgetName { name: "p" },
                    modifiers: vec![],
                    classes: vec![ClassName { name: "h1" }],
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            PropertyName { name: "text" },
                            ExprNode::Term(TermNode::Variable(Variable { name: "text" })),
                        );
                        props.insert(
                            PropertyName { name: "font-size" },
                            ExprNode::Term(TermNode::Constant(ConstantNode::Pixels(24.0))),
                        );
                        props
                    },
                    children: vec![WidgetLayoutNode {
                        widget: WidgetName { name: "span" },
                        modifiers: vec![WidgetModifier::If(ExprNode::Term(TermNode::Variable(
                            Variable { name: "exclaim" }
                        ),))],
                        classes: vec![],
                        properties: {
                            let mut props = HashMap::new();
                            props.insert(
                                PropertyName { name: "color" },
                                ExprNode::Term(TermNode::Constant(ConstantNode::Color(
                                    Color::srgb(0.0, 1.0, 0.0),
                                ))),
                            );
                            props.insert(
                                PropertyName { name: "text" },
                                ExprNode::Term(TermNode::Constant(ConstantNode::String("!"))),
                            );
                            props
                        },
                        children: vec![],
                        output: None,
                    }],
                    output: Some(OutputName::DEFAULT),
                },
            }],
        }
    );
}

#[test]
fn test_simple_style() {
    let code = r#"
            style p +h3 {
                font-size: 18px;
                color: #fff;

                with span +important {
                    font-weight: bold;
                }
            }
        "#;
    let module = parse(code);
    assert_eq!(
        module,
        ModuleNode {
            imports: vec![],
            variables: vec![],
            layouts: vec![],
            styles: vec![StyleNode {
                widget: StyleWidget::Specific(WidgetName { name: "p" }),
                selectors: vec![StyleSelectorNode::WithClass(ClassName { name: "h3" })],
                in_modifier: OutputName::DEFAULT,
                body: StyleBodyNode {
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            PropertyName { name: "font-size" },
                            ExprNode::Term(TermNode::Constant(ConstantNode::Pixels(18.0))),
                        );
                        props.insert(
                            PropertyName { name: "color" },
                            ExprNode::Term(TermNode::Constant(ConstantNode::Color(Color::srgb(
                                1.0, 1.0, 1.0,
                            )))),
                        );
                        props
                    },
                    nested_styles: vec![StyleNode {
                        widget: StyleWidget::Specific(WidgetName { name: "span" }),
                        selectors: vec![StyleSelectorNode::WithClass(ClassName {
                            name: "important"
                        })],
                        in_modifier: OutputName::DEFAULT,
                        body: StyleBodyNode {
                            properties: {
                                let mut props = HashMap::new();
                                props.insert(
                                    PropertyName {
                                        name: "font-weight",
                                    },
                                    ExprNode::Term(TermNode::Constant(ConstantNode::String(
                                        "bold",
                                    ))),
                                );
                                props
                            },
                            nested_styles: vec![],
                        },
                    }],
                }
            }],
            widgets: vec![],
        }
    );
}

#[test]
fn test_global_variables() {
    let code = r#"
        var primaryColor = #00ffff;
        var secondaryColor = #ffff00;
        var paddingSize = 15px;

        const appName = "NekoMaid";
        const maxItems = 100;
    "#;

    let module = parse(code);
    assert_eq!(
        module,
        ModuleNode {
            imports: vec![],
            variables: vec![
                VarAssignNode {
                    variable: Variable {
                        name: "primaryColor"
                    },
                    expr: ExprNode::Term(TermNode::Constant(ConstantNode::Color(Color::srgb(
                        0.0, 1.0, 1.0,
                    )))),
                    var_type: VariableType::Variable,
                },
                VarAssignNode {
                    variable: Variable {
                        name: "secondaryColor"
                    },
                    expr: ExprNode::Term(TermNode::Constant(ConstantNode::Color(Color::srgb(
                        1.0, 1.0, 0.0,
                    )))),
                    var_type: VariableType::Variable,
                },
                VarAssignNode {
                    variable: Variable {
                        name: "paddingSize"
                    },
                    expr: ExprNode::Term(TermNode::Constant(ConstantNode::Pixels(15.0))),
                    var_type: VariableType::Variable,
                },
                VarAssignNode {
                    variable: Variable { name: "appName" },
                    expr: ExprNode::Term(TermNode::Constant(ConstantNode::String("NekoMaid"))),
                    var_type: VariableType::Constant,
                },
                VarAssignNode {
                    variable: Variable { name: "maxItems" },
                    expr: ExprNode::Term(TermNode::Constant(ConstantNode::Number(100.0))),
                    var_type: VariableType::Constant,
                },
            ],
            layouts: vec![],
            styles: vec![],
            widgets: vec![],
        }
    );
}
