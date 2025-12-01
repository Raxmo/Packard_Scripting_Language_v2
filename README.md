# Packard Scripting Language v2

An interactive fiction language designed for writers who want to create branching narratives without deep programming knowledge.

**Current Status**: Parser 100% Complete, Runtime 95% Complete

## Overview

PSL is a Lisp-inspired scripting language with square brackets and keyword parameters, specifically designed for interactive fiction. Writers can define characters, containers of attributes, set variables, create conditional branches, and build interactive choice-driven stories.

## Quick Start

### Build
```bash
cargo build --release
```

### Run a Story
```bash
cargo run --bin packard <story.psl>
```

### Example
```psl
[[define: [character: hero]]:
	[[set: [attribute: name]]: [text: Arion]],
	[[set: [attribute: health]]: [number: 100]]
],

[[chapter: start]: [text: The Beginning]],

[[as: [[from: [character: hero]]: [attribute: name]]]:
[text: I wake up in a strange forest. My health is good.
]],

[[display: [text: What do I do?]]:
	[[option: [text: Explore]]: [goto: [section: explore]]],
	[[option: [text: Rest]]: [goto: [section: rest]]]
],

[section: explore],
[[as:]: [text: I venture deeper into the forest.]],

[section: rest],
[[as:]: [text: I find shelter and rest for the night.]]
```

## Language Features

### Data Types
- **Text**: String values (e.g., `[text: Hello World]`)
- **Number**: Floating point values (e.g., `[number: 42.5]`)
- **Flag**: Boolean values (e.g., `[flag: on]` or `[flag: off]`)
- **Item**: Valueless marker (e.g., `[item:]`)
- **Container**: Nested structures for organizing attributes

### Defining Characters and Attributes
```psl
[[define: [character: main]]:
	[[set: [attribute: name]]: [text: Hero]],
	[[define: [container: inventory]]:
		[[set: [attribute: sword]]: [item:]],
		[[set: [attribute: coins]]: [number: 50]]
	]
]
```

### Accessing and Modifying Values
```psl
// Access: [from: base]: path
[[from: [character: main]]: [attribute: name]]

// Set: [[set: target]: value]
[[set: [[from: [character: main]]: [attribute: name]]]: [text: NewName]]

// Add: [[add: target]: value]
[[add: [[from: [character: main]]: [attribute: coins]]]: [number: 10]]

// Remove: [[remove: target]]
[[remove: [[from: [character: main]]: [attribute: sword]]]]
```

### Story Structure
- **Chapters**: `[[chapter: chapter_name]: [text: Display Text]]`
- **Sections**: `[section: section_name]` - location markers
- **Display**: `[[display: [text: Message]]: options...]` - show text and choices
- **Options**: `[[option: [text: Choice]]: [goto: target]]` - user choices
- **Narrative**: `[[as: label]: [text: Story Text]]` - display text with optional label
- **Goto**: `[goto: [chapter: name]]` or `[goto: [section: name]]` - navigate

### Conditions and Logic
```psl
// If statement: [[if: condition]: body]
[[if: [[[from: [character: main]]: [attribute: coins]] > [number: 0]]]:
	[[display: [text: I can afford supplies!]]:
		[[option: [text: Buy]]: [goto: [section: shop]]]
	]
]

// Exists check: [exists: target]
[[if: [exists: [[from: [character: main]]: [container: bag]]]]:
	[text: I have a bag!]
]
```

### Operators
- **Arithmetic**: `+`, `-`, `*`, `/`
- **Comparison**: `>`, `<`, `>=`, `<=`, `==`, `!=`

## Project Structure

```
src/
  main.rs       - Entry point and output formatting
  lib.rs        - Library exports
  lexer.rs      - Tokenization
  parser.rs     - AST construction
  runtime.rs    - Story execution engine
  value.rs      - Value types and utilities
  error.rs      - Error handling
```

## Output Format

The interpreter outputs:
1. **Story Events**: Chapters, location changes, display messages, narrative text
2. **Options**: Available choices with their navigation targets

Example output:
```
=== STORY OUTPUT ===

[CHAPTER: start] The Beginning
[AS: Hero] I find myself in a strange place.
[DISPLAY] What should I do?

--- OPTIONS ---
  > Explore -> section::forest
  > Rest -> section::camp
```

## Known Limitations

1. **No Save/Load**: Story state isn't persisted between runs
2. **No Interactive Mode**: Currently just outputs all text at once (future enhancement)
3. **Limited Text Formatting**: No bold, italics, or other formatting
4. **No Expressions in Text**: Can't embed `[from: ...]` directly in text strings
5. **No Audio/Visual**: Pure text-based experience

## Testing

Run the baseline test:
```bash
cargo run --bin packard baseline.psl
```

Run a simple test:
```bash
cargo run --bin packard test_simple.psl
```

## Development

The project uses Git Flow branching:
- `main`: Release-ready code
- `develop`: Integration branch for features
- `feature/*`: Feature development branches
- `bugfix/*`: Bug fix branches

See `AGENTS.md` for detailed development guidelines.

## Design Philosophy

1. **Writer-Focused**: Natural syntax for story authors
2. **Accessible**: No programming knowledge required
3. **Clear Over Clever**: Explicit behavior, helpful error messages
4. **Composable**: Complex stories from simple, reusable pieces

## Next Steps

- Build comprehensive test suite
- Interactive mode with user input handling
- Save/load functionality
- Performance optimizations
- Enhanced error messages with line numbers
