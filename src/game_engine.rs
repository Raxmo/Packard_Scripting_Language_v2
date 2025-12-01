use std::io::{self, Write};
use crate::lexer::tokenize;
use crate::parser::{parse, Expr};
use crate::runtime::Runtime;
use crate::error::PslError;
use crate::save_state::GameState;

/// Interactive game engine for PSL stories
pub struct GameEngine {
    program: Vec<Expr>,
    runtime: Runtime,
    current_index: usize,
    story_file: String,
}

impl GameEngine {
    /// Create a new game engine from source code
    pub fn new(source: &str, story_file: &str) -> Result<Self, PslError> {
        let tokens = tokenize(source)?;
        let program = parse(tokens)?;
        
        Ok(GameEngine {
            program,
            runtime: Runtime::new(),
            current_index: 0,
            story_file: story_file.to_string(),
        })
    }

    /// Load a game from a saved state
    pub fn from_save(source: &str, save_path: &str) -> Result<Self, PslError> {
        let tokens = tokenize(source)?;
        let program = parse(tokens)?;
        let saved_state = GameState::load(save_path)?;

        let mut engine = GameEngine {
            program,
            runtime: Runtime::new(),
            current_index: saved_state.current_index,
            story_file: saved_state.metadata.story_file.clone(),
        };

        // Restore runtime state
        engine.runtime.restore_state(
            saved_state.globals,
            saved_state.current_chapter,
            saved_state.current_section,
        )?;

        Ok(engine)
    }

    /// Save the current game state
    pub fn save(&self, save_path: &str) -> Result<(), PslError> {
        let mut state = GameState::new(self.story_file.clone(), "autosave".to_string());
        state.current_index = self.current_index;
        state.globals = self.runtime.get_globals();
        state.current_chapter = self.runtime.get_current_chapter();
        state.current_section = self.runtime.get_current_section();

        state.save(save_path)?;
        Ok(())
    }

    /// Run the game interactively
    pub fn run_interactive(&mut self) -> Result<(), PslError> {
        println!("\n╔════════════════════════════════════════╗");
        println!("║   PACKARD INTERACTIVE FICTION ENGINE   ║");
        println!("╚════════════════════════════════════════╝\n");

        // First execution pass
        loop {
            // Execute until we hit a display statement or reach the end
            if !self.execute_until_display()? {
                // Story ended naturally
                println!("\n[END OF STORY]");
                break;
            }

            // Display output from the story
            self.display_story_output();

            // Get options and let user choose
            let options: Vec<(String, String)> = self.runtime.get_options().to_vec();
            if options.is_empty() {
                println!("\n[END OF STORY]");
                break;
            }

            let choice = self.get_user_choice(&options)?;
            
            // Get the selected option
            let (text, target) = options[choice].clone();
            self.runtime.clear_output();
            println!("\n> {}", text);
            
            // Navigate to the target and reset index
            self.navigate_to_target(&target)?;
        }

        Ok(())
    }

    /// Execute expressions until a display is encountered or story ends
    /// Returns true if display was found, false if story ended
    fn execute_until_display(&mut self) -> Result<bool, PslError> {
        while self.current_index < self.program.len() {
            let expr = self.program[self.current_index].clone();
            
            // Check if this is a display keyword before executing
            let is_display = if let Expr::Keyword { name, .. } = &expr {
                name == "display"
            } else {
                false
            };
            
            self.runtime.eval(&expr)?;
            self.current_index += 1;

            // Stop after executing a display statement
            if is_display {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Navigate to a chapter or section
    fn navigate_to_target(&mut self, target: &str) -> Result<(), PslError> {
        // Parse target
        let location_name = if target.starts_with("chapter::") {
            (true, target.strip_prefix("chapter::").unwrap().to_string())
        } else if target.starts_with("section::") {
            (false, target.strip_prefix("section::").unwrap().to_string())
        } else {
            return Ok(());
        };

        let is_chapter = location_name.0;
        let name = &location_name.1;

        // Find the location in the program
        for (idx, expr) in self.program.iter().enumerate() {
            let matches = if is_chapter {
                matches!(expr, Expr::Chapter(n) if n == name) ||
                matches!(expr, Expr::Keyword { name: kw, params } if kw == "chapter" && 
                    params.iter().any(|(k, v)| k == "id" && matches!(v, Expr::Chapter(n) if n == name)))
            } else {
                matches!(expr, Expr::Section(n) if n == name)
            };

            if matches {
                self.current_index = idx + 1; // Start after the location marker
                return Ok(());
            }
        }

        Ok(())
    }

    /// Display story output in a formatted way
    fn display_story_output(&self) {
        for line in self.runtime.get_output() {
            match line {
                s if s.starts_with("[CHAPTER:") => {
                    // Format chapter headings
                    let content = s.strip_prefix("[CHAPTER: ").unwrap_or("")
                        .strip_suffix("]").unwrap_or("");
                    let parts: Vec<&str> = content.splitn(2, "] ").collect();
                    if parts.len() == 2 {
                        println!("\n╔════════════════════════════════════════╗");
                        println!("║ {:<38} ║", parts[1]);
                        println!("╚════════════════════════════════════════╝\n");
                    }
                }
                s if s.starts_with("[AS:") => {
                    // Format narrative text with labels
                    let content = s.strip_prefix("[AS: ").unwrap_or("")
                        .strip_suffix("]").unwrap_or("");
                    let parts: Vec<&str> = content.splitn(2, "] ").collect();
                    if parts.len() == 2 {
                        println!("{}: {}\n", parts[0], parts[1]);
                    } else {
                        println!("{}\n", content);
                    }
                }
                s if s.starts_with("[TEXT]") => {
                    // Format simple narrative
                    let content = s.strip_prefix("[TEXT] ").unwrap_or("");
                    println!("{}\n", content);
                }
                s if s.starts_with("[DISPLAY]") => {
                    // Format display prompts
                    let content = s.strip_prefix("[DISPLAY] ").unwrap_or("");
                    println!("╭─ {}", content);
                }
                s if s.starts_with("[LOCATION:") => {
                    // Silently track location (don't display)
                }
                _ => println!("{}", line),
            }
        }
    }

    /// Get user choice from available options
    fn get_user_choice(&self, options: &[(String, String)]) -> Result<usize, PslError> {
        loop {
            println!("\n╰─ Choose an option:");
            for (idx, (text, _)) in options.iter().enumerate() {
                println!("   [{}] {}", idx + 1, text);
            }

            print!("\nEnter choice (1-{}): ", options.len());
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();

            if let Ok(choice) = input.trim().parse::<usize>() {
                if choice > 0 && choice <= options.len() {
                    return Ok(choice - 1);
                }
            }

            println!("Invalid choice. Please try again.");
        }
    }
}
