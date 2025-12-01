#[cfg(test)]
mod tests {
    use crate::lexer::tokenize;
    use crate::parser::parse;
    use crate::runtime::Runtime;

    fn run_story(source: &str) -> (Vec<String>, Vec<(String, String)>) {
        let tokens = tokenize(source).expect("Tokenization failed");
        let program = parse(tokens).expect("Parsing failed");
        let mut runtime = Runtime::new();
        runtime.execute(program).expect("Runtime failed");
        
        let output = runtime.get_output().to_vec();
        let options = runtime.get_options().to_vec();
        (output, options)
    }

    #[test]
    fn test_character_definition() {
        let source = r#"
[[define: [character: alice]]:
	[[set: [attribute: name]]: [text: Alice]]
],
[[as:]: [text: Character defined]]
        "#;
        let (output, _) = run_story(source);
        assert!(output.iter().any(|s| s.contains("Character defined")), "Should produce output");
    }

    #[test]
    fn test_chapter_display() {
        let source = r#"
[[chapter: start]: [text: Chapter One]]
        "#;
        let (output, _) = run_story(source);
        assert!(output.iter().any(|s| s.contains("Chapter One")), "Should display chapter");
    }

    #[test]
    fn test_narrative_text() {
        let source = r#"
[[as:]: [text: This is a story]]
        "#;
        let (output, _) = run_story(source);
        assert!(output.iter().any(|s| s.contains("This is a story")), "Should display text");
    }

    #[test]
    fn test_narrative_with_label() {
        let source = r#"
[[define: [character: hero]]:
	[[set: [attribute: name]]: [text: Arion]]
],
[[as: [[from: [character: hero]]: [attribute: name]]]: [text: A story]]
        "#;
        let (output, _) = run_story(source);
        assert!(output.iter().any(|s| s.contains("AS: Arion")), "Should display with label");
    }

    #[test]
    fn test_display_and_options() {
        let source = r#"
[[display: [text: What next]]:
	[[option: [text: Option A]]: [goto: [section: a]]],
	[[option: [text: Option B]]: [goto: [section: b]]]
]
        "#;
        let (output, options) = run_story(source);
        assert!(output.iter().any(|s| s.contains("What next")), "Should display message");
        assert_eq!(options.len(), 2, "Should have 2 options");
        assert_eq!(options[0].0, "Option A", "Option text correct");
        assert_eq!(options[1].0, "Option B", "Option text correct");
    }

    #[test]
    fn test_goto_chapter() {
        let source = r#"
[[display: [text: Go?]]:
	[[option: [text: Go]]: [goto: [chapter: next]]]
]
        "#;
        let (_, options) = run_story(source);
        assert!(options[0].1.contains("chapter::next"), "Goto should target chapter");
    }

    #[test]
    fn test_goto_section() {
        let source = r#"
[[display: [text: Go?]]:
	[[option: [text: Go]]: [goto: [section: next]]]
]
        "#;
        let (_, options) = run_story(source);
        assert!(options[0].1.contains("section::next"), "Goto should target section");
    }

    #[test]
    fn test_section_location() {
        let source = r#"
[section: mysection]
        "#;
        let (output, _) = run_story(source);
        assert!(output.iter().any(|s| s.contains("section:mysection")), "Should mark section location");
    }

    #[test]
    fn test_set_and_get_attribute() {
        let source = r#"
[[define: [character: alice]]:
	[[set: [attribute: coins]]: [number: 100]]
],
[[set: [[from: [character: alice]]: [attribute: coins]]]: [number: 50]]
        "#;
        let (_, _) = run_story(source);
        // If no error, the test passed
    }

    #[test]
    fn test_add_attribute() {
        let source = r#"
[[define: [character: alice]]:
	[[set: [attribute: coins]]: [number: 100]]
],
[[add: [[from: [character: alice]]: [attribute: coins]]]: [number: 50]]
        "#;
        let (_, _) = run_story(source);
        // If no error, the test passed
    }

    #[test]
    fn test_container_definition() {
        let source = r#"
[[define: [character: alice]]:
	[[define: [container: inventory]]:
		[[set: [attribute: sword]]: [item:]]
	]
]
        "#;
        let (_, _) = run_story(source);
        // If no error, the test passed
    }

    #[test]
    fn test_arithmetic_addition() {
        let source = r#"
[[define: [character: alice]]:
	[[set: [attribute: coins]]: [number: 100]]
],
[[if: [[[from: [character: alice]]: [attribute: coins]] > [number: 50]]]:
	[[as:]: [text: Rich]]
]
        "#;
        let (output, _) = run_story(source);
        assert!(output.iter().any(|s| s.contains("Rich")), "Should execute if with arithmetic");
    }

    #[test]
    fn test_comparison_greater_than() {
        let source = r#"
[[define: [character: alice]]:
	[[set: [attribute: level]]: [number: 10]]
],
[[if: [[[from: [character: alice]]: [attribute: level]] > [number: 5]]]:
	[[as:]: [text: High level]]
]
        "#;
        let (output, _) = run_story(source);
        assert!(output.iter().any(|s| s.contains("High level")), "Should handle comparison");
    }

    #[test]
    fn test_false_condition() {
        let source = r#"
[[if: [flag: off]]:
	[[as:]: [text: This should not appear]]
]
        "#;
        let (output, _) = run_story(source);
        assert!(!output.iter().any(|s| s.contains("should not appear")), "Should skip false condition");
    }

    #[test]
    fn test_remove_attribute() {
        let source = r#"
[[define: [character: alice]]:
	[[set: [attribute: sword]]: [item:]]
],
[[remove: [[from: [character: alice]]: [attribute: sword]]]]
        "#;
        let (_, _) = run_story(source);
        // If no error, the test passed
    }

    #[test]
    fn test_nested_paths() {
        let source = r#"
[[define: [character: alice]]:
	[[define: [container: bag]]:
		[[set: [attribute: coins]]: [number: 100]]
	]
]
        "#;
        let (_, _) = run_story(source);
        // If no error, the test passed
    }

    #[test]
    fn test_empty_container() {
        let source = r#"
[[define: [character: alice]]:
	[[define: [container: empty]]]
]
        "#;
        let (_, _) = run_story(source);
        // If no error, the test passed
    }

    #[test]
    fn test_text_attribute() {
        let source = r#"
[[define: [character: alice]]:
	[[set: [attribute: name]]: [text: Alice Wonderland]]
]
        "#;
        let (_, _) = run_story(source);
        // If no error, the test passed
    }

    #[test]
    fn test_flag_attribute() {
        let source = r#"
[[define: [character: alice]]:
	[[set: [attribute: alive]]: [flag: on]],
	[[set: [attribute: asleep]]: [flag: off]]
]
        "#;
        let (_, _) = run_story(source);
        // If no error, the test passed
    }
}
