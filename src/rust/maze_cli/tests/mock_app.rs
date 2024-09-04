extern crate maze;
extern crate maze_cli;

use maze_cli::app::App;
use maze::LinePrinter;
use maze::Maze;
use maze::Definition;

use std::collections::VecDeque;
use std::error::Error;
use std::io::{self};

struct MockInputKey {
    key: char,
    reset_output: bool,
}

struct MockInputLine {
    text: String,
    reset_output: bool,
}

pub struct MockApp {
    input_keys: VecDeque<MockInputKey>,
    input_lines: VecDeque<MockInputLine>,
    output: Vec<String>,
    pub current_maze: Maze,
    current_maze_name: String,
}

impl MockApp {
    pub fn new() -> MockApp {
        MockApp {
            input_keys: VecDeque::new(),
            input_lines: VecDeque::new(),
            output: Vec::new(),
            current_maze: Maze::new(Definition::new(0, 0)),
            current_maze_name: "".to_string(),
        }
    }

    pub fn add_input_key(&mut self, key: char, reset_output: bool) {
        self.input_keys.push_back(MockInputKey {
            key,
            reset_output,
        });
    }

    pub fn add_input_line(&mut self, text: &str, reset_output: bool) {
        self.input_lines.push_back(MockInputLine {
            text: text.to_string(),
            reset_output,
        });
    }

    pub fn io_error(message: String) -> io::Error {
        io::Error::new(io::ErrorKind::Other, message)
    }

    pub fn verify_output(&self, expected: Vec<&str>) -> Result<(), io::Error> {
        if expected.len() != self.output.len() {
            return Err(Self::io_error(
                format!("The output and expected lines differ in length. Expected has length {}, while output has length {}.",
                expected.len(),
                self.output.len() )                    
            ));
        }
        for (i, expected_line) in expected.iter().enumerate() {
            if *expected_line != self.output[i] {
                return Err(Self::io_error(
                    format!("Difference found in output at index {}: expected[{}] = '{}', output[{}] = '{}'", i, i, expected_line, i, self.output[i])
                ));
            }
        }
        Ok(())
    }

    #[cfg(feature = "print_output")]
    pub fn print_output(&self) {
        println!("Captured output:\n");
        for line in &self.output {
            println!("{}", line);
        }
    }
}

impl Default for MockApp {
    fn default() -> Self {
             Self::new()
         }
     }

impl App for MockApp {
    fn get_maze(&self) -> &Maze {
        &self.current_maze
    }

    fn get_maze_mut(&mut self) -> &mut Maze {
        &mut self.current_maze
    }

    fn get_maze_name(&self) -> String { 
        self.current_maze_name.clone()
    }

    fn set_maze_name(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        self.current_maze_name = name.to_string();
        Ok(())
    }

    fn read_key(&mut self) -> Result<Option<char>, io::Error> {
        match self.input_keys.pop_front() {
            Some(input_key) => {
                if input_key.reset_output {
                    self.output.clear();
                }
                Ok(Some(input_key.key))
            }
            None => Err(Self::io_error("No key presses found in input_keys buffer".to_string())),
        }
    }

    fn read_line(&mut self) -> Result<Option<String>, io::Error> {
        match self.input_lines.pop_front() {
            Some(input_line) => {
                if input_line.reset_output {
                    self.output.clear();
                }
                Ok(Some(input_line.text))
            }
            None => Err(Self::io_error("No lines found in input_lines buffer".to_string())),
        }
    }
    
    fn get_line_printer(&mut self) -> &mut dyn LinePrinter {
        self
    }    
}

impl LinePrinter for MockApp {
    fn print_line(&mut self, line: &str) -> Result<(), io::Error> {
        self.output.push(line.to_string());
        Ok(())
    }
}
