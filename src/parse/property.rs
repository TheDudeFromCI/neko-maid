//! A module for parsing and handling properties in NekoMaid UI files.

use std::fmt;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::parse::NekoMaidParseError;
use crate::parse::context::{NekoResult, ParseContext};
use crate::parse::token::TokenType;
use crate::parse::value::PropertyValue;

/// A property within a style or element.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Property {
    /// The name of the property.
    pub(super) name: String,

    /// The value of the property.
    pub(super) value: PropertyValue,
}

/// A property within a style or element.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct UnresolvedProperty {
    /// The name of the property.
    pub(super) name: String,

    /// The value of the property.
    pub(super) value: UnresolvedPropertyValue,
}

/// An unresolved property value that may be a constant or a variable reference.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum UnresolvedPropertyValue {
    /// A constant property value.
    Constant(PropertyValue),

    /// A variable reference.
    Variable(String),
}

impl UnresolvedPropertyValue {
    /// Resolves the property value using the provided variable map.
    pub(super) fn resolve(
        &self,
        variables: &HashMap<String, PropertyValue>,
    ) -> NekoResult<PropertyValue> {
        match self {
            UnresolvedPropertyValue::Constant(v) => Ok(v.clone()),
            UnresolvedPropertyValue::Variable(var_name) => {
                if let Some(v) = variables.get(var_name) {
                    Ok(v.clone())
                } else {
                    Err(NekoMaidParseError::VariableNotFound {
                        variable: var_name.clone(),
                        position: Default::default(),
                    })
                }
            }
        }
    }
}

/// The type of a widget property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyType {
    /// A string type.
    String,

    /// A numeric type.
    Number,

    /// A boolean type.
    Boolean,

    /// A color type.
    Color,

    /// A percentage type.
    Percentage,

    /// A pixel type.
    Pixels,
}

impl fmt::Display for PropertyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = match self {
            PropertyType::String => "string",
            PropertyType::Number => "number",
            PropertyType::Boolean => "boolean",
            PropertyType::Color => "color",
            PropertyType::Percentage => "percentage",
            PropertyType::Pixels => "pixels",
        };
        write!(f, "{}", type_name)
    }
}

/// Parses a property from the input and returns a [`Property`].
pub(super) fn parse_property(ctx: &mut ParseContext) -> NekoResult<Property> {
    let name = ctx.expect_as_string(TokenType::Identifier)?;
    ctx.expect(TokenType::Colon)?;
    let value = parse_value(ctx)?;
    ctx.expect(TokenType::Semicolon)?;

    Ok(Property { name, value })
}

/// Parses an unresolved property from the input and returns a
/// [`UnresolvedProperty`].
pub(super) fn parse_unresolved_property(ctx: &mut ParseContext) -> NekoResult<UnresolvedProperty> {
    let name = ctx.expect_as_string(TokenType::Identifier)?;
    ctx.expect(TokenType::Colon)?;
    let value = parse_unresolved_value(ctx)?;
    ctx.expect(TokenType::Semicolon)?;

    Ok(UnresolvedProperty { name, value })
}

/// Parses a variable declaration from the input and returns a [`Property`].
pub(super) fn parse_variable(ctx: &mut ParseContext) -> NekoResult<Property> {
    ctx.expect(TokenType::VarKeyword)?;
    let name = ctx.expect_as_string(TokenType::Identifier)?;
    ctx.expect(TokenType::Equals)?;
    let value = parse_value(ctx)?;
    ctx.expect(TokenType::Semicolon)?;

    Ok(Property { name, value })
}

/// Parses an unresolved property value from the input and returns a
/// [`UnresolvedPropertyValue`].
pub(super) fn parse_unresolved_value(
    ctx: &mut ParseContext,
) -> NekoResult<UnresolvedPropertyValue> {
    let next_pos = ctx.next_position().unwrap_or_default();
    let next = ctx.consume()?;

    match next.token_type {
        TokenType::Identifier | TokenType::StringLiteral => Ok(UnresolvedPropertyValue::Constant(
            next.into_string_property(next_pos)?,
        )),
        TokenType::ColorLiteral => Ok(UnresolvedPropertyValue::Constant(
            next.into_color_property(next_pos)?,
        )),
        TokenType::BooleanLiteral => Ok(UnresolvedPropertyValue::Constant(
            next.into_boolean_property(next_pos)?,
        )),
        TokenType::NumberLiteral => Ok(UnresolvedPropertyValue::Constant(
            next.into_number_property(next_pos)?,
        )),
        TokenType::PercentLiteral => Ok(UnresolvedPropertyValue::Constant(
            next.into_percent_property(next_pos)?,
        )),
        TokenType::PixelsLiteral => Ok(UnresolvedPropertyValue::Constant(
            next.into_pixels_property(next_pos)?,
        )),
        TokenType::Variable => {
            let var_name = next.into_variable_name(next_pos)?;
            Ok(UnresolvedPropertyValue::Variable(var_name))
        }
        _ => Err(NekoMaidParseError::UnexpectedToken {
            expected: vec![
                TokenType::StringLiteral.type_name().to_string(),
                TokenType::Identifier.type_name().to_string(),
                TokenType::ColorLiteral.type_name().to_string(),
                TokenType::BooleanLiteral.type_name().to_string(),
                TokenType::NumberLiteral.type_name().to_string(),
                TokenType::PercentLiteral.type_name().to_string(),
                TokenType::PixelsLiteral.type_name().to_string(),
                TokenType::Variable.type_name().to_string(),
            ],
            found: format!("{}", next.token_type),
            position: next.position,
        }),
    }
}

/// Parses a property value from the input and returns a [`PropertyValue`].
pub(super) fn parse_value(ctx: &mut ParseContext) -> NekoResult<PropertyValue> {
    let next_pos = ctx.next_position().unwrap_or_default();
    let value = parse_unresolved_value(ctx)?;
    match value {
        UnresolvedPropertyValue::Constant(v) => Ok(v),
        UnresolvedPropertyValue::Variable(var_name) => match ctx.get_variable(&var_name) {
            Some(v) => Ok(v.clone()),
            None => Err(NekoMaidParseError::VariableNotFound {
                variable: var_name,
                position: next_pos,
            }),
        },
    }
}
