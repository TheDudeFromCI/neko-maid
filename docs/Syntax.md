# NekoMaid UI Syntax

## Comment Lines

Comment lines can be used to write additional information down in your code that
will be ignored by the compiler.

```nekomaid
// This message will be ignored.

layout p; // Comments can also be placed on the same line as regular code.
```

## Constants

There are several default data types defined in NekoMaid. All of these have literal definitions which can be used to define constant values within the code.

### Strings

String literals can be defined using double quotes (`"`), single quotes (`'`), or backticks (`` ` ``). NekoMaid does not currently support escaped characters or unicode characters.

**Examples:**

```nekomaid
"Hello, world!"
'Cats are awesome.'
`Some third string.`
```

### Numbers

Numbers can be defined as either integers or floats. (Decimal points must be prefixed by a digit.)

**Examples:**

```nekomaid
12
-42.0
0.13

// This will fail to compile.
// .1
```

#### Pixels / Percentage

Pixels and Percentages are special types of numbers, ending with a `px` or `%` suffix, respectively.

**Examples:**

```nekomaid
50px
75.3%
```

### Boolean

Booleans are values that can be either `true` or `false`.

**Examples:**

```nekomaid
true
false
```

### Color

Hex colors can be defined directly in NekoMaid by using a hashtag (`#`) followed by a number of hexadecimal numbers. A color may contain either 3, 4, 6, or 8 hex characters.

Letters may be either uppercase or lowercase.

**Examples:**

```nekomaid
// shorthand for #aabbcc
// defines an srgb color using Color_u8(170, 187, 204)
#abc

// shorthand for #aabbccdd
// defines an srgba color using Color_u8(170, 187, 204, 221)
#abcd

// defines an srgb color using Color_u8(1, 171, 35)
#01ab23

// defines an srgba color using Color_u8(171, 18, 205, 52)
#ab12cd34
```

### List

Lists are ordered arrays of values separated by `,` and enclosed using `[]`. These values do not have to be the same type.

**Examples:**

```nekomaid
[ 1, "apple", 23px, [`another list`, #abc] ]

// You can also define them across multiple lines
[
  1, 2, 3, 4,
  5, 6, 7, 8,  // trailing commas are allowed
]
```

### Dict

A dict, short for dictionary, is a data type that contains a set of unordered property-value pairs, defined using the format `property: value`. These pairs are separated by `,` and enclosed using `{}`

Property tags are alpha-numeric values, supporting underscores and dashes, however, all properties may not *start* with a number or a dash.

**Examples:**

```nekomaid
{ color: #fff, width: 50% }

// You can also define them across multiple lines, if you need.
{
  fruit: "apple" // notice that properties are not enclosed in strings
}

// this will error; properties cannot start with a number.
{ 2d-width: 10px }
```

## Variables

Variables can be used to store temporary data that can be reused to determine how the layout is rendered.

## Layouts

A layout is a tree of UI elements that are spawned to the screen. By creating a layout, this will cause a new element, or "widget" of the given type to be spawned to the screen.

A layout can be created by using the `layout` keyword, followed by name of the widget that should be created.

```nekomaid
layout div;
```

After creating a widget, properties can be assigned to that widget by replacing the `;` with a property block.

```nekomaid
layout div {
  width: 100px;
  height: 50px;
}
```

Properties can be assigned using property names and assigning values to them, separated by a `:`. Lines must end in a `;`.

Valid property names are alpha-numeric values with support for dashes and underscores. Does not support leading numbers or dashes.

### Nested Elements

In the properties block of a layout,

### Modifiers

## Widgets

### Native Widgets

### Custom Widgets

## Styles
