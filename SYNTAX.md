# Packard Scripting Language (PSL) Syntax Specification

## Overview
PSL is an interactive fiction language using S-expression syntax with square brackets and keyword-tagged parameters. It prioritizes writer accessibility and learnability.

## Core Syntax Elements

### Expressions
All PSL code consists of expressions in the form:
```
[keyword: parameter1, parameter2, ...]
```

### Brackets
- Square brackets `[]` are used for all expressions
- Proper nesting is required
- Comments use `//` for single-line

## Data Types

### text
String values. Implied string content (no quotes needed).
```
[text: This is a string value]
```

### number
Numeric values (integers and floats).
```
[number: 42]
[number: 3.14]
```

### flag
Boolean values: `on` or `off`.
```
[flag: on]
[flag: off]
```

### item
Empty placeholder type indicating the existence of something.
```
[item:]
```

## Declarations

### Character Definition
```
[[define: [character: character_name]]:
  attributes and containers...
]
```

### Container Definition
Nested within characters to hold attributes.
```
[[define: [container: container_name]]:
  attributes...
]
```

### Attribute Setting
```
[[set: [attribute: name]]: value]
```

## Narrative Structure

### Chapter
Section marker with display text. Clears screen and waits for user input.
```
[[chapter: chapter_id]: [text: Display text]]
```

### Section
Navigation point. Clears screen but no automatic display.
```
[section: section_id]
```

### Display
Shows text and processes following option tags.
```
[[display: [text: Display text]]:
  options...
]
```

### Option
Provides user with a choice.
```
[[option: [text: Choice text]]: action]
```

## Control Flow

### Goto
Navigate to a chapter or section.
```
[goto: [chapter: chapter_id]]
[goto: [section: section_id]]
```

### Conditional (if)
Execute based on condition.
```
[[if: condition]:
  action
]
```

### Conditions
- `[exists: path]` - Check if attribute exists
- Comparison operators: `>`, `<`, `>=`, `<=`, `==`, `!=`

## Data Access

### From
Access nested attributes via path.
```
[from: [character: main]: [attribute: name]]
[from: [from: [character: main]: [container: bag]]: [attribute: sword]]
```

Path nesting: `[from: ... value]` where value can be another `[from: ...]`

## Operations

### Set
Assign a value to an attribute. Supports expressions on right side.
```
[[set: [attribute: name]]: [text: New Value]]
[[set: path]: arithmetic_expression]
```

### Remove
Delete an attribute from its container.
```
[[remove: path_to_attribute]]
```

### Add
Add a new attribute to a container.
```
[[add: path]: [item:]]
```

### Arithmetic
Supported operations: `+`, `-`, `*`, `/`
```
[[[from: path]: [attribute: money]] - [number: 5]]
```

## Display Modifiers

### As
Title text with a source (e.g., character name).
```
[[as: [[from: [character: main]]: [attribute: name]]]:
  [text: Dialogue text]
]
```

Use `[[as:]` without source for unnamed text.

## Examples

### Simple Character Setup
```
[[define: [character: hero]]:
  [[set: [attribute: name]]: [text: The Hero]],
  [[define: [container: inventory]]:
    [[set: [attribute: gold]]: [number: 100]]
  ]
]
```

### Navigation
```
[[chapter: intro]: [text: Welcome!]],
[[display: [text: What next?]]:
  [[option: [text: Continue]]: [goto: [section: main_story]]]
],
[section: main_story],
[[as:]: [text: The adventure begins...]]
```

### Conditional Logic
```
[[display: [text: Options]]:
  [[if: [exists: [[from: [character: hero]: [container: inventory]]: [attribute: sword]]]]:
    [[option: [text: Fight]]: [goto: [section: battle]]]
  ],
  [[if: [[[from: [character: hero]: [container: inventory]]: [attribute: gold]] > [number: 50]]]:
    [[option: [text: Shop]]: [goto: [section: shop]]]
  ]
]
```

## Comments
Single-line comments use `//`
```
// This is a comment
[[define: [character: main]]:  // Character definition
  [[set: [attribute: name]]: [text: Hero]]
]
```

## Design Rationale

### Why Square Brackets?
- Clear visual distinction from parentheses
- Writer-friendly appearance
- Reduces visual confusion with nested structures

### Why Keyword Tags?
- Explicit parameter purpose (self-documenting code)
- Reduces cognitive load for writers
- Easier to parse and understand at a glance

### Why No Quotes for Strings?
- Simpler syntax for writers
- Type annotations (`[text: ...]`) make intent clear
- Reduces visual noise
